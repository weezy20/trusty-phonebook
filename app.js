const express = require("express");
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
app.set("json spaces", 2);
//  The json-parser functions so that it takes the JSON data of a request,
// transforms it into a JavaScript object and then attaches it to the body property of the request object
// before the route handler is called.
app.use(express.json());

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
    return rs.status(400).json({ error: "Content missing" });
  }
  const note = {
    content: body.content,
    important: body.important || false,
    date: body.date || new Date(),
    id: newId,
  };

  console.log("Adding note ->\n ", note);
  notes.push(note);
  rs.json(note);
});

module.exports = app;
