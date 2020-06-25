/*
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
 *
 *  This file is part of Seni
 *
 *  Seni is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Seni is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

///<reference path='Log.ts'/>
///<reference path='Matrix.ts'/>

enum TextureUnit {
    renderToTexture = 0,
    brushTexture,
    maskTexture,
}


enum RPCommand {
    geometry = 1,
    mask,
    image,
}

class SketchShaderTS {
    public program: WebGLProgram;

    public positionAttribute: GLint;
    public colourAttribute: GLint;
    public textureAttribute: GLint;
    public pMatrixUniform: WebGLUniformLocation | null;
    public brushUniform: WebGLUniformLocation | null;
    public maskUniform: WebGLUniformLocation | null;
    public canvasDimUniform: WebGLUniformLocation | null;
    public maskInvert: WebGLUniformLocation | null;

    // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
    // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
    //
    public outputLinearColourSpaceUniform: WebGLUniformLocation | null;


    constructor(program: WebGLProgram,
                positionAttribute: GLint,
                colourAttribute: GLint,
                textureAttribute: GLint,
                pMatrixUniform: WebGLUniformLocation | null,
                brushUniform: WebGLUniformLocation | null,
                maskUniform: WebGLUniformLocation | null,
                canvasDimUniform: WebGLUniformLocation | null,
                maskInvert: WebGLUniformLocation | null,
                outputLinearColourSpaceUniform: WebGLUniformLocation | null) {
        this.program = program;
        this.positionAttribute = positionAttribute;
        this.colourAttribute = colourAttribute;
        this.textureAttribute = textureAttribute;
        this.pMatrixUniform = pMatrixUniform;
        this.brushUniform = brushUniform;
        this.maskUniform = maskUniform;
        this.canvasDimUniform = canvasDimUniform;
        this.maskInvert = maskInvert;
        this.outputLinearColourSpaceUniform = outputLinearColourSpaceUniform;
    }
}

class BlitShaderTS {
    public program: WebGLProgram;

    public positionAttribute: GLint;
    public textureAttribute: GLint;
    public pMatrixUniform: WebGLUniformLocation | null;
    public textureUniform: WebGLUniformLocation | null;
    // older versions of seni (pre 4.2.0) did not convert from sRGB space to linear before blending
    // in order to retain the look of these older sketchs we can't carry out the linear -> sRGB conversion
    //
    public outputLinearColourSpaceUniform: WebGLUniformLocation | null;

    public brightnessUniform: WebGLUniformLocation | null;
    public contrastUniform: WebGLUniformLocation | null;
    public saturationUniform: WebGLUniformLocation | null;

    constructor(program: WebGLProgram,
                positionAttribute: GLint,
                textureAttribute: GLint,
                pMatrixUniform: WebGLUniformLocation | null,
                textureUniform: WebGLUniformLocation | null,
                outputLinearColourSpaceUniform: WebGLUniformLocation | null,
                brightness: WebGLUniformLocation | null,
                contrast: WebGLUniformLocation | null,
                saturation: WebGLUniformLocation | null) {
        this.program = program;
        this.positionAttribute = positionAttribute;
        this.textureAttribute = textureAttribute;
        this.pMatrixUniform = pMatrixUniform;
        this.textureUniform = textureUniform;
        this.outputLinearColourSpaceUniform = outputLinearColourSpaceUniform;
        this.brightnessUniform = brightness;
        this.contrastUniform = contrast;
        this.saturationUniform = saturation;

    }
}

class RendererTS {
    static initGL(canvas: HTMLCanvasElement): WebGLRenderingContext | null {

        // todo: reinstate the try/catch block ???

        //        try {
        const gl = canvas.getContext('webgl', {
            alpha: false,
            preserveDrawingBuffer: true
        });
        // commented out because of jshint
        //    if (!gl) {
        //alert('Could not initialise WebGL, sorry :-(');
        //    }

        return gl;

        // } catch (e) {
        //     return undefined;
        // }
    }

    static compileShader(gl: WebGLRenderingContext, shaderType: number, src: string) {
        Log.log("compileShader called");
        Log.log(src);
        const shader = gl.createShader(shaderType);

        if (shader) {
            gl.shaderSource(shader, src);
            gl.compileShader(shader);

            if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
                const shaderInfoLog = gl.getShaderInfoLog(shader);
                if (shaderInfoLog) {
                    Log.log(shaderInfoLog);
                }
                gl.deleteShader(shader);
                return null;
            }
            return shader;
        }

        return null;
    }

    static setupSketchShaders(gl: WebGLRenderingContext, vertexSrc: string, fragmentSrc: string): SketchShaderTS | null {
        const programOrNull: WebGLProgram | null = gl.createProgram();

        if (programOrNull) {
            const program: WebGLProgram = programOrNull;

            const vertexShader = RendererTS.compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
            const fragmentShader = RendererTS.compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

            if (vertexShader && fragmentShader) {
                gl.attachShader(program, vertexShader);
                gl.attachShader(program, fragmentShader);

                gl.linkProgram(program);

                if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
                    let lastError = gl.getProgramInfoLog(program);

                    alert(`Could not initialise shaders: ${lastError}`);;
                    gl.deleteProgram(program);
                    return null;
                }

                return new SketchShaderTS(program,
                                          gl.getAttribLocation(program, 'pos'),
                                          gl.getAttribLocation(program, 'col'),
                                          gl.getAttribLocation(program, 'uv'),
                                          gl.getUniformLocation(program, 'proj_matrix'),
                                          gl.getUniformLocation(program, 'brush'),
                                          gl.getUniformLocation(program, 'mask'),
                                          gl.getUniformLocation(program, 'canvas_dim'),
                                          gl.getUniformLocation(program, 'mask_invert'),
                                          gl.getUniformLocation(program, 'output_linear_colour_space'));
            }
        }

        return null;
    }

    static setupBlitShaders(gl: WebGLRenderingContext, vertexSrc: string, fragmentSrc: string): BlitShaderTS | null {
        Log.log("setupBlitShaders");
        const programOrNull: WebGLProgram | null = gl.createProgram();

        if (programOrNull) {
            const program: WebGLProgram = programOrNull;

            const vertexShader = RendererTS.compileShader(gl, gl.VERTEX_SHADER, vertexSrc);
            const fragmentShader = RendererTS.compileShader(gl, gl.FRAGMENT_SHADER, fragmentSrc);

            if (vertexShader && fragmentShader) {
                gl.attachShader(program, vertexShader);
                gl.attachShader(program, fragmentShader);

                gl.linkProgram(program);

                if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
                    let lastError = gl.getProgramInfoLog(program);

                    alert(`Could not initialise shaders: ${lastError}`);;
                    gl.deleteProgram(program);
                    return null;
                }

                return new BlitShaderTS(program,
                                        gl.getAttribLocation(program, 'pos'),
                                        gl.getAttribLocation(program, 'uv'),
                                        gl.getUniformLocation(program, 'proj_matrix'),
                                        gl.getUniformLocation(program, 'rendered_image'),
                                        gl.getUniformLocation(program, 'output_linear_colour_space'),
                                        gl.getUniformLocation(program, 'brightness'),
                                        gl.getUniformLocation(program, 'contrast'),
                                        gl.getUniformLocation(program, 'saturation'));
            }
        }

        Log.log("setupBlitShaders RETURNING NULL");
        return null;
    }

    static setupGLState(gl: WebGLRenderingContext) {
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

    static textureUnitToGl(gl: WebGLRenderingContext, unit: TextureUnit): GLint {
        switch(unit) {
            case TextureUnit.renderToTexture: return gl.TEXTURE0;
            case TextureUnit.brushTexture: return gl.TEXTURE1;
            case TextureUnit.maskTexture: return gl.TEXTURE2;
            default:
                Log.error(`invalid unit for texture: ${unit}`);
        };
        return gl.TEXTURE0;
    }

    static createRenderTexture(gl: WebGLRenderingContext, renderTextureWidth: number, renderTextureHeight: number): WebGLTexture | null {
        let texUnit = RendererTS.textureUnitToGl(gl, TextureUnit.renderToTexture);
        Log.log(`activeTexture ${TextureUnit.renderToTexture}`);
        gl.activeTexture(texUnit);

        const targetTexture = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, targetTexture);

        {
            // define size and format of level 0
            const level = 0;
            const internalFormat = gl.RGBA;
            const border = 0;
            const format = gl.RGBA;
            const g_type = gl.UNSIGNED_BYTE;
            const data: any = null;
            gl.texImage2D(gl.TEXTURE_2D, level, internalFormat,
                          renderTextureWidth, renderTextureHeight, border,
                          format, g_type, data);

            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            // gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
            // gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        }

        return targetTexture;
    }

    static createFrameBuffer(gl: WebGLRenderingContext, targetTexture: WebGLTexture): WebGLFramebuffer | null {
        // Create and bind the framebuffer
        const fb = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

        // attach the texture as the first color attachment
        const attachmentPoint = gl.COLOR_ATTACHMENT0;
        const level = 0;
        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachmentPoint, gl.TEXTURE_2D, targetTexture, level);

        return fb;
    }

    static checkFramebufferStatus(gl: WebGLRenderingContext) {
        let res = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
        switch(res) {
            case gl.FRAMEBUFFER_COMPLETE: Log.log("gl.FRAMEBUFFER_COMPLETE"); break;
            case gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT: Log.log("gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT"); break;
            case gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: Log.log("gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"); break;
            case gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS: Log.log("gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS"); break;
            case gl.FRAMEBUFFER_UNSUPPORTED: Log.log("gl.FRAMEBUFFER_UNSUPPORTED"); break;

                // todo: these two values can be returned if using a webgl2 context
                // (see https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/checkFramebufferStatus)
                // commented out because the emacs/typescript setup doesn't recognise them as part of gl
            // case gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: Log.log("gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE"); break;
            // case gl.RENDERBUFFER_SAMPLES: Log.log("gl.RENDERBUFFER_SAMPLE"); break;
        }
    }

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
    static getProjectionMatrixExtents(canvasDim: number, sectionDim: number, section: number): [number, number, number, number] {
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
}

interface IHashStrStr {
    [key: string] : string;
}

interface IHashStrTextureUnit {
    [key: string] : WebGLTexture;
}

class GLRenderer2 {
    public glDomElement: HTMLCanvasElement;
    public gl: WebGLRenderingContext | null;
    public loadedTextureCache: IHashStrTextureUnit;
    public sketchShader: SketchShaderTS | null;
    public blitShader: BlitShaderTS | null;
    public glVertexBuffer: WebGLBuffer | null;
    public pMatrix: Float32Array;
    public renderTexture: WebGLTexture | null;
    public framebuffer: WebGLFramebuffer | null;

    constructor(prefix: string, canvasElement: HTMLCanvasElement, shaders: IHashStrStr, renderTextureWidth: number, renderTextureHeight: number) {
        this.glDomElement = canvasElement;

        // webgl setup
        const gl = RendererTS.initGL(this.glDomElement);
        this.gl = gl;

        if (gl) {
            // map of texture filename -> texture unit
            this.loadedTextureCache = {};

            // note: constructors can't be async so the shaders should already have been loaded by loadShaders
            this.sketchShader = RendererTS.setupSketchShaders(gl,
                                                              shaders[prefix + '/shader/main-vert.glsl'],
                                                              shaders[prefix + '/shader/main-frag.glsl']);
            this.blitShader = RendererTS.setupBlitShaders(gl,
                                                          shaders[prefix + '/shader/blit-vert.glsl'],
                                                          shaders[prefix + '/shader/blit-frag.glsl']);

            RendererTS.setupGLState(gl);

            this.glVertexBuffer = gl.createBuffer();

            // this.mvMatrix = Matrix.create();
            this.pMatrix = Matrix.create();

            this.renderTexture = RendererTS.createRenderTexture(gl, renderTextureWidth, renderTextureHeight);
            if (this.renderTexture) {
                this.framebuffer = RendererTS.createFrameBuffer(gl, this.renderTexture);
            }


            // render to the canvas
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        }
    }

    // isg: used in sketch.html?
    public clear() {
        if (this.gl) {
            this.gl.clear(this.gl.COLOR_BUFFER_BIT | this.gl.DEPTH_BUFFER_BIT);
        }
    }

    public loadImage(src: string) {
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

    // todo: this is copied from stuff.js, use that version instead
    normalize_bitmap_url(prefix: string, url: string) {
        const re = /^[\w-/]+.png/;

        if (url.match(re)) {
            // requesting a bitmap just by filename, so get it from /img/immutable/
            return prefix + "/img/immutable/" + url;
        } else {
            // change nothing, try and fetch the url
            return url;
        }
    }

    public async ensureTexture(unit: TextureUnit, prefix: string, src: string) {
        // console.log(`ensureTexture called with ${prefix} ${src}`)
        let normalized_src = this.normalize_bitmap_url(prefix, src);
        if (this.gl) {
        let texUnit = RendererTS.textureUnitToGl(this.gl, unit);

        if (this.loadedTextureCache[normalized_src] === undefined) {
            // console.log(`ensureTexture loading: ${normalized_src}`);
            let image: ImageData = <ImageData>await this.loadImage(normalized_src);

            let gl = this.gl;

            const texture = gl.createTexture();
            if (texture) {
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
        }

        const texture = this.loadedTextureCache[normalized_src];
        this.gl.activeTexture(texUnit);
            this.gl.bindTexture(this.gl.TEXTURE_2D, texture);
        }
    }

    public async renderGeometryToTexture(prefix: string, meta: any, destTextureWidth: number, destTextureHeight: number, memoryF32: any, buffers: any, sectionDim: number, section: number) {
        if (this.gl === null) {
            return;
        }
        if (this.framebuffer === null) {
            return;
        }
        if (this.sketchShader === null) {
            return;
        }

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

        let [left, right, bottom, top] = RendererTS.getProjectionMatrixExtents(canvasDim, sectionDim, section);
        Matrix.ortho(this.pMatrix, left, right, bottom, top, 10, -10);

        gl.uniformMatrix4fv(shader.pMatrixUniform,
                            false,
                            this.pMatrix);

        gl.uniform1i(shader.brushUniform, TextureUnit.brushTexture);
        gl.uniform1i(shader.maskUniform, TextureUnit.maskTexture);

        // setting output_linear_colour_space in meta because the blit shader also requires it
        meta.output_linear_colour_space = false;
        // the contrast/brightness/saturation values are only used by the blit shader
        meta.contrast = 1.0;
        meta.brightness = 0.0;
        meta.saturation = 1.0;
        gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);
        gl.uniform1i(shader.maskInvert, 0);

        gl.uniform1f(shader.canvasDimUniform, canvasDim);

        const glVertexBuffer = this.glVertexBuffer;

        const bytesin32bit = 4;

        const vertexItemSize = 2;
        const colourItemSize = 4;
        const textureItemSize = 2;
        const totalSize = (vertexItemSize + colourItemSize + textureItemSize);

        await this.ensureTexture(TextureUnit.maskTexture, prefix, 'mask/white.png');

        for(let b = 0; b < buffers.length; b++) {
            let buffer = buffers[b];
            switch(buffer.command) {
                case RPCommand.geometry:
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
                case RPCommand.mask:
                    await this.ensureTexture(TextureUnit.maskTexture, prefix, buffer.mask_filename);
                    gl.uniform1i(shader.maskInvert, buffer.mask_invert);
                    break;
                case RPCommand.image:

                    meta.output_linear_colour_space = buffer.linearColourSpace;
                    meta.contrast = buffer.contrast;
                    meta.brightness = buffer.brightness;
                    meta.saturation = buffer.saturation;

                    gl.uniform1i(shader.outputLinearColourSpaceUniform, meta.output_linear_colour_space);
                    // todo(isg): apply the image modifications in the blit shader
                    break;
                default:
                    Log.error(`unknown RenderPacket command ${buffer.command}`);
            }
        }
    }

    public clearBuffer() {
        if (!this.gl) {
            return;
        }

        const gl = this.gl;

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        // gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    }

    public renderTextureToScreen(meta: any, canvasWidth: number, canvasHeight: number) {
        if (!this.gl || !this.framebuffer || !this.blitShader) {
            return;
        }

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

        gl.uniform1i(shader.textureUniform, TextureUnit.renderToTexture);

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

        // const bytesin32bit = 4;

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

    public copyImageDataTo(elem: HTMLImageElement) {
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

    public localDownload(filename: string) {
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
