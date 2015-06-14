
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

var galleryData = [
  makeGalleryItem(1, '14ef-blur-grid', '14ef-blur-grid.png'),
  makeGalleryItem(2, '14eg-marker-grid', '14eg-marker-grid.png'),
  makeGalleryItem(3, '14eh-chaotic-grid', '14eh-chaotic-grid.png'),
  makeGalleryItem(4, '14fh-rothko-1', '14fh-rothko-1.png'),
  makeGalleryItem(5, '151c-flower', '151c-flower.png'),
  makeGalleryItem(6, '1531-four-squares', '1531-four-squares.png'),
  makeGalleryItem(7, '154c-stroked-bezier', '154c-stroked-bezier.png'),
  makeGalleryItem(8, '154h-rothko-2', '154h-rothko-2.png'),
  makeGalleryItem(9, '1556-biomorphs', '1556-biomorphs.png'),
  makeGalleryItem(10, '155b-spiral-derived', '155b-spiral-derived.png'),
  makeGalleryItem(11, '155j-chaotic-grid-2', '155j-chaotic-grid-2.png'),
  makeGalleryItem(12, '155j-chaotic-grid-3', '155j-chaotic-grid-3.png'),
  makeGalleryItem(13, '1560-path', '1560-path.png'),
  makeGalleryItem(14, '1565-hex-grid', '1565-hex-grid.png'),
  makeGalleryItem(15, '156h-tri-grid', '156h-tri-grid.png'),
  makeGalleryItem(16, '157e-tile-shadow', '157e-tile-shadow.png'),
  makeGalleryItem(17, '1580-quilt', '1580-quilt.png'),
  makeGalleryItem(18, '1585-seeds', '1585-seeds.png')
].reverse();

app.set('port', (process.env.PORT || 3000));

app.use('/', express.static(path.join(__dirname, '..')));
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
