const express = require("express");
const app = express();

// Step 1 : return a hardcoded list of phonebook entries on
// http://localhost:3001/api/persons

const PORT = 3001;
app.set("json spaces", 2);
let phonebook = require("./phonebook");
// console.log(phonebook)

app.get("/api/persons", (req, res) => {
  res.json(phonebook);
});

// Step 2: http://localhost:3001/info
// The page has to show the time that the request was received
// and how many entries are in the phonebook at the time of processing the request.
app.get("/info", (req, res) => {
  const dateCurrent = new Date();
  const personsNumber = phonebook.length;
  const response = `<p>Phonebook has info for ${personsNumber} people</p>
    <p>${dateCurrent}</p>`;
  res.send(response);
});

// Step 3:
// Implement the functionality for displaying the information for a single phonebook entry.
// The url for getting the data for a person with the id 5 should be
// http://localhost:3001/api/persons/5
app.get("/api/persons/:id", (req, res) => {
  const id = Number(req.params.id);
  const person = phonebook.find((person) => person.id === id);
  if (!person) {
    res.statusMessage = `Person id ${id} does not exist in phonebook`;
    res.status(404).end();
  }
  res.send(person);
});
// Step: 4
// Implement functionality that makes it possible to delete a single phonebook entry
// by making an HTTP DELETE request to the unique URL of that phonebook entry.
app.delete("/api/persons/:id", (req, res) => {
  const id = Number(req.params.id);
  const deletedPerson = phonebook.find((person) => person.id === id);
  if (!deletedPerson) {
    // what do?
  }
  console.log("Person deleted", deletedPerson);
  phonebook = phonebook.filter((person) => person.id !== id);
  res.status(204).end();
});

// Step 5:
// Expand the backend so that new phonebook entries can be added by making HTTP POST
// requests to the address http://localhost:3001/api/persons.

// Generate a new id for the phonebook entry with the Math.random function.
// Use a big enough range for your random values so that the likelihood of
// creating duplicate ids is small.
app.use(express.json());
app.post("/api/persons", (req, res) => {
  const personInfo = req.body;
  const missingError = !personInfo.name
    ? !personInfo.number
      ? "Name and Number missing"
      : "Name missing"
    : !personInfo.number
    ? "Number missing"
    : "None";
  if (missingError !== "None")
    return res.status(400).json({ error: missingError });
  // step 6 : since we already did step 6 first half here, we will do the remaining
  // ie a pre-existing name entry should be rejected
  if (checkNameAlreadyExists(personInfo.name))
    return res
      .status(400)
      .json({ error: "Name must be unique! (I know this is stupid but)" });

  console.log(personInfo);
  const personId = generatePersonId();
  personInfo.id = personId;
  phonebook = phonebook.concat(personInfo);
  res.statusMessage = "Person entry created";
  res.status(204).end();
});

app.listen(PORT, () => console.log("Server running at localhost:" + PORT));
// This initialization works because of the event loop, the code doesn't
// block at app.listen .., and allows code after it to execute
let sampleSpace = 100;
function generatePersonId() {
  let candidate = genRandomInt(sampleSpace);
  // check for duplicates
  while (phonebook.find((person) => person.id === candidate)) {
    // On each try where we get a collision, extend the range
    initialSpace *= 10;
    candidate = genRandomInt(sampleSpace);
  }
  return candidate;
}
const genRandomInt = (max) => 1 + Math.floor(max * Math.random());

function checkNameAlreadyExists(name) {
  return phonebook.find((p) => p.name == name);
}
/*
https://stackoverflow.com/questions/1226810/is-http-post-request-allowed-to-send-back-a-response-body
The action performed by the POST method might not result in a resource that can be
 identified by a URI. In this case, either 200 (OK) or 204 (No Content) is the appropriate 
 response status, depending on whether or not the response includes an entity that 
 describes the result.

If a resource has been created on the origin server,
 the response SHOULD be 201 (Created) and contain an entity which describes the 
 status of the request and refers to the new resource, and a Location header 
 (see section 14.30).
*/
