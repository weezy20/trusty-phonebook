const express = require("express");
// Es6 way ->
const requestLogger = require("./my_request_logger.js");
let notes = [
  {
    id: 0,
    content: "This is express server",
    date: "2019-05-30T17:30:31.098Z",
    important: true,
  },
  {
    id: 1,
    content: "HTML is easy and this is express",
    date: "2019-05-30T17:30:31.098Z",
    important: true,
  },
  {
    id: 2,
    content: "Browser can execute only Javascript",
    date: "2019-05-30T18:39:34.091Z",
    important: false,
  },
  {
    id: 3,
    content: "GET and POST are the most important methods of HTTP protocol",
    date: "2019-05-30T19:20:14.298Z",
    important: true,
  },
];
const app = express();
const cors = require("cors");
app.set("json spaces", 2);
// The json-parser functions so that it takes the JSON data of a request,
// transforms it into a JavaScript object and then attaches it to the body property of the request object
// before the route handler is called.
// *---------------------------------------------------------------------------------*
// Notice that json-parser is taken into use before the requestLogger middleware,
// because otherwise request.body will not be initialized when the logger is executed!

// Theory :
// Middleware functions have to be taken into use before routes if we want them to be executed before the route event handlers
// are called. There are also situations where we want to define middleware functions after routes.
// In practice, this means that we are defining middleware functions that are only called if no route handles the HTTP request.
app.use(cors());
app.use(express.json());
app.use(requestLogger);

app.get("/", (rq, rs) => {
  rs.send("<h1>Hello World</h1>");
});

app.get("/notes", (rq, rs) => {
  rs.json(notes);
});

app.get("/notes/:id", (rq, rs) => {
  const id = Number(rq.params.id);
  // Indexing with [id] won't work in the long run because think about what happens when DELETE is added to the mix
  // The indexes can shift in an unpredictable fashion
  // const note = notes[id];
  // note ? rs.json(note) : rs.status(404).end()
  const note = notes.find((note) => note.id === id);
  if (note) {
    rs.json(note);
  } else {
    rs.statusMessage = `Note id ${id} does not exist`;
    rs.status(404).end();
  }
});

app.delete("/notes/:id", (rq, rs) => {
  const id = Number(rq.params.id);
  const deletedNote = notes.find((note) => note.id == id);
  notes = notes.filter((note) => note.id !== id);
  // 204 - return no data with the response. two options are 204 and 404
  rs.json(deletedNote).status(204).end();
});

// A mock route to test POST requests
app.post("/mock/notes", (rq, rs) => {
  const note = rq.body;
  console.log("Adding mock note ->\n ", note);
  rs.json(note);
});

app.post("/notes", (rq, rs) => {
  const body = rq.body;
  // https://stackoverflow.com/questions/44934828/is-it-spread-syntax-or-the-spread-operator
  // new Id for a note should be +1 the last max that exists in the notes object
  const newId =
    notes.length > 0 ? 1 + Math.max(...notes.map((note) => note.id)) : 0;
  if (!body.content) {
    // 400 Bad request
    return rs.status(400).json({ error: "Content missing" });
  }
  const note = {
    content: body.content,
    important: body.important || false,
    date: new Date(), // Don't rely on the client to supply a date
    id: newId,
  };

  console.log("Adding note ->\n ", note);
  // notes.push(note);
  // or
  notes = notes.concat(note);
  rs.json(note);
});

// Notice that we define this handler and `use` it in express at the bottom, after
// all the routes have been registered. This ensures that this middleware is run only
// after the express server has fallen through all the previous middleware/routes
const unknownEndpoint = (_request, response) => {
  response.status(404).send({ error: "unknown endpoint" });
};
// Register the above handler, the handler may be defined anywhere of course
// but we are using () => {} syntax so we define it just before we use it
app.use(unknownEndpoint);

module.exports = app;
