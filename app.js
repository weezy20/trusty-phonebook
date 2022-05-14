const express = require('express')
let notes = [
  {
    id: 0,
    content: 'This is express server',
    date: '2019-05-30T17:30:31.098Z',
    important: true,
  },
  {
    id: 1,
    content: 'HTML is easy and this is express',
    date: '2019-05-30T17:30:31.098Z',
    important: true,
  },
  {
    id: 2,
    content: 'Browser can execute only Javascript',
    date: '2019-05-30T18:39:34.091Z',
    important: false,
  },
  {
    id: 3,
    content: 'GET and POST are the most important methods of HTTP protocol',
    date: '2019-05-30T19:20:14.298Z',
    important: true,
  },
]
const app = express()
app.set('json spaces', 2)

app.get('/', (rq, rs) => {
  rs.send('<h1>Hello World</h1>')
})

app.get('/notes', (rq, rs) => {
  rs.json(notes)
})

app.get('/notes/:id', (rq, rs) => {
  const id = Number(rq.params.id);
  // Indexing with [id] won't work in the long run because think about what happens when DELETE is added to the mix
  // The indexes can shift in an unpredictable fashion
  // const note = notes[id];
  // note ? rs.json(note) : rs.status(404).end()
  const note = notes.find(note => note.id === id);
  if (note) {
    rs.json(note)
  } else {
    rs.statusMessage = `Note id ${id} does not exist`
    rs.status(404).end()
  }
})

app.delete('/notes/:id', (rq, rs) => {
    const id = Number(rq.params.id);
    notes = notes.filter(note => note.id !== id);
    // 204 - return no data with the response. two options are 204 and 404
    rs.status(204).end();
})

// app.post('/notes')

module.exports = app
