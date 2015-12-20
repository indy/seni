
var fs = require('fs');
var path = require('path');
var express = require('express');
var bodyParser = require('body-parser');
var app = express();

function makeGalleryItem(id, name, image) {
  return {
    id: id,
    name: name,
    image: 'img/seni/' + image
  };
}

function buildGalleryData() {
  var items = [['14ef-blur-grid', '14ef-blur-grid.png'],
               ['14eg-marker-grid', '14eg-marker-grid.png'],
               ['14eh-chaotic-grid', '14eh-chaotic-grid.png'],
               ['14fh-rothko-1', '14fh-rothko-1.png'],
               ['151c-flower', '151c-flower.png'],
               ['1531-four-squares', '1531-four-squares.png'],
               ['154c-stroked-bezier', '154c-stroked-bezier.png'],
               ['154h-rothko-2', '154h-rothko-2.png'],
               ['1556-biomorphs', '1556-biomorphs.png'],
               ['155b-spiral-derived', '155b-spiral-derived.png'],
               ['155j-chaotic-grid-2', '155j-chaotic-grid-2.png'],
               ['155j-chaotic-grid-3', '155j-chaotic-grid-3.png'],
               ['1560-path', '1560-path.png'],
               ['1565-hex-grid', '1565-hex-grid.png'],
               ['156h-tri-grid', '156h-tri-grid.png'],
               ['157e-tile-shadow', '157e-tile-shadow.png'],
               ['1580-quilt', '1580-quilt.png'],
               ['1585-seeds', '1585-seeds.png'],
               ['15fe-rotate-mirror', '15fe-rotate-mirror.png'],
               ['15h3-polychrome', '15h3-polychrome.png'],
               ['15h3-rotate-mirror-2', '15h3-rotate-mirror-2.png'],
               ['15h3-tri-grid', '15h3-tri-grid.png'],
               ['15h4-chromatic-layers-1', '15h4-chromatic-layers-1.png'],
               ['15h4-chromatic-layers-2', '15h4-chromatic-layers-2.png'],
               ['15h4-chromatic-layers-3', '15h4-chromatic-layers-3.png'],
               ['15h4-rotate-mirror', '15h4-rotate-mirror.png'],
               ['15h4-mirror-layers', '15h4-mirror-layers.png'],
               ['15he-rotate-mirror', '15he-rotate-mirror.png'],
               ['15he-mirror-layers', '15he-mirror-layers.png'],
               ['15he-cos-1', '15he-cos-1.png'],
               ['15he-cos-2', '15he-cos-2.png'],
               ['15he-cos-3', '15he-cos-3.png']
              ];

  var res = [];
  for(var i = 0; i < items.length; i++) {
    res.push(makeGalleryItem(i+1, items[i][0], items[i][1]));
  }

  return res.reverse();
}

var galleryData = buildGalleryData();

app.set('port', (process.env.PORT || 3000));

app.use('/', express.static(path.join(__dirname, '..', 'app')));
app.use('/node_modules', express.static(path.join(__dirname, '..', 'node_modules')));
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({extended: true}));

app.listen(app.get('port'), function() {
  console.log('Server started: http://localhost:' + app.get('port') + '/');
});

app.get('/gallery', function(req, res) {
  res.setHeader('Content-Type', 'application/json');
  res.send(JSON.stringify(galleryData));
});

function getPieceFilename(id) {

  var piece = null;

  for(var i = 0;i < galleryData.length; i++) {
    if(galleryData[i].id === id) {
      piece = galleryData[i];
      return path.join('seni', piece.name + '.seni');
    }
  }
  return undefined;
}

app.get('/gallery/:pieceId', function(req, res) {
  res.setHeader('Content-Type', 'application/json');
  var pieceId = parseInt(req.params.pieceId, 10);
  var filename = getPieceFilename(pieceId);
  if(!filename) {
    res.send('cannot find piece with id of ' + pieceId);
    return;
  }

  fs.readFile(filename, function(err, code) {
    res.send(code);
  });
});

/*
app.post('/comments.json', function(req, res) {
  fs.readFile('comments.json', function(err, data) {
    var comments = JSON.parse(data);
    comments.push(req.body);
    fs.writeFile('comments.json', JSON.stringify(comments, null, 4), function(err) {
      res.setHeader('Content-Type', 'application/json');
      res.setHeader('Cache-Control', 'no-cache');
      res.send(JSON.stringify(comments));
    });
  });
});
*/
