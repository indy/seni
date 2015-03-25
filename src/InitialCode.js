const code = `

(define coords (pair 440 400
               533 700
                   766 200
                   900 500))

(spline colour: (col/rgb r: 0.0 g: 0.4 b: 0.1 a: 0.5)
        lineWidthStart:10
        lineWidthEnd: 0
        tStart: 0.0
        tEnd: 0.5
        coords: (pair 140 600
                      333 800
                      566 700))

(bezier tessellation: 35
    lineWidth: 10
     coords: coords
     colour: (col/rgb r: 0.0 g: 0.0 b: 0.1 a: 0.5))

(bezierScratch tessellation: 2
          lineWidth: 10
          coords: coords
          strokeWidth: 10
          strokeTessellation: 5
          strokeNoise: 125
          colour: (col/rgb r: 0.2 g: 0.9 b: 0.1 a: 0.3)
          colourVolatility: 0
          seed: 43)

`;

const InitialCode = {
  getCode: () => code
};

export default InitialCode;
