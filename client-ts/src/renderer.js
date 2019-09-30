// --------------------------------------------------------------------------------
// renderer

const TEXTURE_UNIT_RENDER_TO_TEXTURE = 0;
const TEXTURE_UNIT_BRUSH_TEXTURE = 1;
const TEXTURE_UNIT_MASK_TEXTURE = 2;

const RPCommand_Geometry = 1;
const RPCommand_Mask = 2;
const RPCommand_Image = 3;

function initGL(canvas) {
  try {
    const gl = canvas.getContext('webgl', {
      alpha: false,
      preserveDrawingBuffer: true
    });
    // commented out because of jshint
    //    if (!gl) {
    //alert('Could not initialise WebGL, sorry :-(');
    //    }

    return gl;

  } catch (e) {
    return undefined;
  }
}

function compileShader(gl, type, src) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, src);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.log(gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

function setupSketchShaders(gl, vertexSrc, fragmentSrc) {
  const shader = {};

  shader.program = gl.createProgram();

  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

  gl.attachShader(shader.program, vertexShader);
  gl.attachShader(shader.program, fragmentShader);

  gl.linkProgram(shader.program);

  if (!gl.getProgramParameter(shader.program, gl.LINK_STATUS)) {
    let lastError = gl.getProgramInfoLog(shader.program);

    alert(`Could not initialise shaders: ${lastError}`);;
    gl.deleteProgram(shader.program);
    return null;
  }

  shader.positionAttribute = gl.getAttribLocation(shader.program, 'pos');
  shader.colourAttribute = gl.getAttribLocation(shader.program, 'col');
  shader.textureAttribute = gl.getAttribLocation(shader.program, 'uv');
  shader.pMatrixUniform = gl.getUniformLocation(shader.program, 'proj_matrix');
  shader.brushUniform = gl.getUniformLocation(shader.program, 'brush');
  shader.maskUniform = gl.getUniformLocation(shader.program, 'mask');
  shader.canvasDimUniform = gl.getUniformLocation(shader.program, 'canvas_dim');
  shader.maskInvert = gl.getUniformLocation(shader.program, 'mask_invert');

  // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
  // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
  //
  shader.outputLinearColourSpaceUniform = gl.getUniformLocation(shader.program, 'output_linear_colour_space');

  return shader;
}

function setupBlitShaders(gl, vertexSrc, fragmentSrc) {
  const shader = {};

  shader.program = gl.createProgram();

  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

  gl.attachShader(shader.program, vertexShader);
  gl.attachShader(shader.program, fragmentShader);

  gl.linkProgram(shader.program);

  if (!gl.getProgramParameter(shader.program, gl.LINK_STATUS)) {
    let lastError = gl.getProgramInfoLog(shader.program);

    alert(`Could not initialise shaders: ${lastError}`);;
    gl.deleteProgram(shader.program);
    return null;
  }

  shader.positionAttribute = gl.getAttribLocation(shader.program, 'pos');
  shader.textureAttribute = gl.getAttribLocation(shader.program, 'uv');
  shader.pMatrixUniform = gl.getUniformLocation(shader.program, 'proj_matrix');
  shader.textureUniform  = gl.getUniformLocation(shader.program, 'rendered_image');

  // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
  // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
  //
  shader.outputLinearColourSpaceUniform = gl.getUniformLocation(shader.program, 'output_linear_colour_space');

  shader.brightnessUniform = gl.getUniformLocation(shader.program, 'brightness');
  shader.contrastUniform = gl.getUniformLocation(shader.program, 'contrast');
  shader.saturationUniform = gl.getUniformLocation(shader.program, 'saturation');

  return shader;
}

function setupGLState(gl) {
  // clear colour alpha is 1.0 as we want to treat a blank canvas as opaque white
  gl.clearColor(1.0, 1.0, 1.0, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  // assuming that we'll be using pre-multiplied alpha
  // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
  gl.enable(gl.BLEND);
  gl.blendEquation(gl.FUNC_ADD);
  gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);

//  gl.disable(gl.DEPTH_TEST);
}

function textureUnitToGl(gl, unit) {
  let texUnit = gl.TEXTURE0 + unit;
  switch(unit) {
  case TEXTURE_UNIT_RENDER_TO_TEXTURE: texUnit = gl.TEXTURE0; break;
  case TEXTURE_UNIT_BRUSH_TEXTURE: texUnit = gl.TEXTURE1; break;
  case TEXTURE_UNIT_MASK_TEXTURE: texUnit = gl.TEXTURE2; break;
  case 3: texUnit = gl.TEXTURE3; break;
  case 4: texUnit = gl.TEXTURE4; break;
  case 5: texUnit = gl.TEXTURE5; break;
  case 6: texUnit = gl.TEXTURE6; break;
  case 7: texUnit = gl.TEXTURE7; break;
  default:
    console.error(`invalid unit for texture: ${unit}`);
  };

  return texUnit;
}

function createRenderTexture(gl, config) {
  // create to render to
  const targetTextureWidth = config.render_texture_width;
  const targetTextureHeight = config.render_texture_height;

  let texUnit = textureUnitToGl(gl, TEXTURE_UNIT_RENDER_TO_TEXTURE);
  console.log(`activeTexture ${TEXTURE_UNIT_RENDER_TO_TEXTURE}`);
  gl.activeTexture(texUnit);

  const targetTexture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, targetTexture);

  {
    // define size and format of level 0
    const level = 0;
    const internalFormat = gl.RGBA;
    const border = 0;
    const format = gl.RGBA;
    const type = gl.UNSIGNED_BYTE;
    const data = null;
    gl.texImage2D(gl.TEXTURE_2D, level, internalFormat,
                  targetTextureWidth, targetTextureHeight, border,
                  format, type, data);

    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    // gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    // gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  return targetTexture;
}

function createFrameBuffer(gl, targetTexture) {
  // Create and bind the framebuffer
  const fb = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

  // attach the texture as the first color attachment
  const attachmentPoint = gl.COLOR_ATTACHMENT0;
  const level = 0;
  gl.framebufferTexture2D(gl.FRAMEBUFFER, attachmentPoint, gl.TEXTURE_2D, targetTexture, level);

  return fb;
}

function checkFramebufferStatus(gl) {
  let res = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
  switch(res) {
  case gl.FRAMEBUFFER_COMPLETE: console.log("gl.FRAMEBUFFER_COMPLETE"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT: console.log("gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: console.log("gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS: console.log("gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS"); break;
  case gl.FRAMEBUFFER_UNSUPPORTED: console.log("gl.FRAMEBUFFER_UNSUPPORTED"); break;
  case gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: console.log("gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE"); break;
  case gl.RENDERBUFFER_SAMPLES: console.log("gl.RENDERBUFFER_SAMPLE"); break;
  }
}

const gConfig = {
  render_texture_width: 2048,
  render_texture_height: 2048,
};

/*
  sectionDim = number of sections along each axis
  section = current section to render

  e.g. sectionDim: 3 ==

  |---+---+---|
  | 0 | 1 | 2 |
  |---+---+---|
  | 3 | 4 | 5 |
  |---+---+---|
  | 6 | 7 | 8 |
  |---+---+---|

  Note: sections begin in the top left corner
  (compared to the canvas co-ordinates which begin in the lower left corner)
*/
function getProjectionMatrixExtents(canvasDim, sectionDim, section) {
  const pictureWidth = canvasDim;
  const pictureHeight = canvasDim;

  const sectionWidth = pictureWidth / sectionDim;
  const sectionHeight = pictureHeight / sectionDim;

  const sectionX = section % sectionDim;
  const sectionY = (sectionDim - Math.floor(section / sectionDim)) - 1;

  let left = sectionX * sectionWidth;
  let right = left + sectionWidth;

  let bottom = sectionY * sectionHeight;
  let top = bottom + sectionHeight;

  return [left, right, bottom, top];
}

class GLRenderer {
  constructor(canvasElement, shaders) {
    this.glDomElement = canvasElement;

    // webgl setup
    const gl = initGL(this.glDomElement);
    this.gl = gl;

    // map of texture filename -> texture unit
    this.loadedTextureCache = {};

    // note: constructors can't be async so the shaders should already have been loaded by loadShaders
    this.sketchShader = setupSketchShaders(gl, shaders['shader/main-vert.glsl'], shaders['shader/main-frag.glsl']);
    this.blitShader = setupBlitShaders(gl, shaders['shader/blit-vert.glsl'], shaders['shader/blit-frag.glsl']);

    setupGLState(gl);

    this.glVertexBuffer = gl.createBuffer();

    // this.mvMatrix = Matrix.create();
    this.pMatrix = Matrix.create();

    this.renderTexture = createRenderTexture(gl, gConfig);
    this.framebuffer = createFrameBuffer(gl, this.renderTexture);

    // render to the canvas
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  }

  // isg: used in sketch.html?
  clear() {
    this.gl.clear(this.gl.COLOR_BUFFER_BIT | this.gl.DEPTH_BUFFER_BIT);
  }

  loadImage(src) {
    let that = this;

    return new Promise((resolve, reject) => {
      const image = new Image();

      image.addEventListener('load', () => {
        resolve(image);
      });

      image.addEventListener('error', () => {
        reject();
      });

      image.src = src;
    });
  }

  async ensureTexture(unit, src) {
    let normalized_src = normalize_bitmap_url(src);
    let texUnit = textureUnitToGl(this.gl, unit);

    if (this.loadedTextureCache[normalized_src] === undefined) {
      // console.log(`ensureTexture loading: ${normalized_src}`);
      let image = await this.loadImage(normalized_src);

      let gl = this.gl;

      const texture = gl.createTexture();

      gl.activeTexture(texUnit);

      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER,
                       gl.LINEAR_MIPMAP_NEAREST);
      gl.generateMipmap(gl.TEXTURE_2D);

      this.loadedTextureCache[normalized_src] = texture;
    }

    const texture = this.loadedTextureCache[normalized_src];
    this.gl.activeTexture(texUnit);
    this.gl.bindTexture(this.gl.TEXTURE_2D, texture);
  }

  async renderGeometryToTexture(meta, destTextureWidth, destTextureHeight, memoryF32, buffers, sectionDim, section) {
    const gl = this.gl;

    let shader = this.sketchShader;

    // render to texture attached to framebuffer

    gl.bindFramebuffer(gl.FRAMEBUFFER, this.framebuffer);
    //gl.bindFramebuffer(gl.FRAMEBUFFER, null);

    gl.viewport(0, 0, destTextureWidth, destTextureHeight);

    gl.useProgram(shader.program);

    gl.enableVertexAttribArray(shader.positionAttribute);
    gl.enableVertexAttribArray(shader.colourAttribute);
    gl.enableVertexAttribArray(shader.textureAttribute);

    // gl.clearColor(0.0, 0.0, 1.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // todo: get the canvasDim from the Rust side
    const canvasDim = 1024.0;

    let [left, right, bottom, top] = getProjectionMatrixExtents(canvasDim, sectionDim, section);
    Matrix.ortho(this.pMatrix, left, right, bottom, top, 10, -10);

    gl.uniformMatrix4fv(shader.pMatrixUniform,
                        false,
                        this.pMatrix);

    gl.uniform1i(shader.brushUniform, TEXTURE_UNIT_BRUSH_TEXTURE);
    gl.uniform1i(shader.maskUniform, TEXTURE_UNIT_MASK_TEXTURE);

    // setting output_linear_colour_space in meta because the blit shader also requires it
    meta.output_linear_colour_space = false;
    // the contrast/brightness/saturation values are only used by the blit shader
    meta.contrast = 1.0;
    meta.brightness = 0.0;
    meta.saturation = 1.0;
    gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);
    gl.uniform1i(shader.maskInvert, false);

    gl.uniform1f(shader.canvasDimUniform, canvasDim);

    const glVertexBuffer = this.glVertexBuffer;

    const bytesin32bit = 4;

    const vertexItemSize = 2;
    const colourItemSize = 4;
    const textureItemSize = 2;
    const totalSize = (vertexItemSize + colourItemSize + textureItemSize);

    await this.ensureTexture(TEXTURE_UNIT_MASK_TEXTURE, 'mask/white.png');

    for(let b = 0; b < buffers.length; b++) {
      let buffer = buffers[b];
      switch(buffer.command) {
      case RPCommand_Geometry:
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray#Syntax
        // a new typed array view is created that views the specified ArrayBuffer
        const gbuf = new Float32Array(memoryF32, buffer.geo_ptr, buffer.geo_len);

        gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, gbuf, gl.STATIC_DRAW);

        gl.vertexAttribPointer(shader.positionAttribute,
                               vertexItemSize,
                               gl.FLOAT, false, totalSize * bytesin32bit,
                               0);
        gl.vertexAttribPointer(shader.colourAttribute,
                               colourItemSize,
                               gl.FLOAT, false, totalSize * bytesin32bit,
                               vertexItemSize * bytesin32bit);
        gl.vertexAttribPointer(shader.textureAttribute,
                               textureItemSize,
                               gl.FLOAT, false, totalSize * bytesin32bit,
                               (vertexItemSize + colourItemSize) * bytesin32bit);

        gl.drawArrays(gl.TRIANGLE_STRIP, 0, buffer.geo_len / totalSize);
        break;
      case RPCommand_Mask:
        await this.ensureTexture(TEXTURE_UNIT_MASK_TEXTURE, buffer.mask_filename);
        gl.uniform1i(shader.maskInvert, buffer.mask_invert);
        break;
      case RPCommand_Image:

        meta.output_linear_colour_space = buffer.linearColourSpace;
        meta.contrast = buffer.contrast;
        meta.brightness = buffer.brightness;
        meta.saturation = buffer.saturation;

        gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);
        // todo(isg): apply the image modifications in the blit shader
        break;
      default:
        console.error(`unknown RenderPacket command ${command}`);
      }
    }
  }

  renderTextureToScreen(meta, canvasWidth, canvasHeight) {
    const gl = this.gl;
    const domElement = this.glDomElement;

    if (domElement.width !== canvasWidth) {
      domElement.width = canvasWidth;
    }
    if (domElement.height !== canvasHeight) {
      domElement.height = canvasHeight;
    }

    let shader = this.blitShader;

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);

    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);

    gl.useProgram(shader.program);

    // console.log(shader);
    gl.enableVertexAttribArray(shader.positionAttribute);
    gl.enableVertexAttribArray(shader.textureAttribute);

    // gl.clearColor(0.0, 0.0, 1.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // render the entirety of the scene
    Matrix.ortho(this.pMatrix, 0, canvasWidth, 0, canvasHeight, 10, -10);

    // add some uniforms for canvas width and height

    gl.uniformMatrix4fv(shader.pMatrixUniform,
                        false,
                        this.pMatrix);

    gl.uniform1i(shader.textureUniform, TEXTURE_UNIT_RENDER_TO_TEXTURE);

    gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);

    gl.uniform1f(shader.brightnessUniform, meta.brightness);
    gl.uniform1f(shader.contrastUniform, meta.contrast);
    gl.uniform1f(shader.saturationUniform, meta.saturation);

    const glVertexBuffer = this.glVertexBuffer;

    // x, y, u, v
    const jsData = [
      0.0, 0.0, 0.0, 0.0,
      canvasWidth, 0.0, 1.0, 0.0,
      0.0, canvasHeight, 0.0, 1.0,
      canvasWidth, canvasHeight, 1.0, 1.0
    ];
    const data = Float32Array.from(jsData);


    const bytesin32bit = 4;

    const vertexItemSize = 2;
    const textureItemSize = 2;
    const totalSize = (vertexItemSize + textureItemSize);

    gl.bindBuffer(gl.ARRAY_BUFFER, glVertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

    gl.vertexAttribPointer(shader.positionAttribute,
                           vertexItemSize,
                           gl.FLOAT, false, totalSize * 4,
                           0);
    gl.vertexAttribPointer(shader.textureAttribute,
                           textureItemSize,
                           gl.FLOAT, false, totalSize * 4,
                           (vertexItemSize) * 4);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, jsData.length / totalSize);
  }

  copyImageDataTo(elem) {
    return new Promise((resolve, reject) => {
      try {
        this.glDomElement.toBlob(blob => {
          // console.log(this.glDomElement);
          // console.log(blob);
          elem.src = window.URL.createObjectURL(blob);
          return resolve();
        });
      } catch (error) {
        return reject(error);
      }
    });
  }

  localDownload(filename) {
    this.glDomElement.toBlob(function(blob) {

      const url = window.URL.createObjectURL(blob);

      let element = document.createElement('a');
      element.setAttribute('href', url);
      // element.setAttribute('target', '_blank');
      element.setAttribute('download', filename);

      element.style.display = 'none';
      document.body.appendChild(element);

      element.click();

      document.body.removeChild(element);

      downloadDialogHide();
    });
  }

}
