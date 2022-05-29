import React from "react";
// import ReactDOM from 'react-dom'
import { useEffect, useState } from "react";
import axios from "axios";
const base_url = "http://localhost";
// TODO: Reject duplicate names from being added, this can be done if the server returns an error

export default function App() {
  // Hook for search bar
  const [searchName, setSearchName] = useState("");
  // Tracks the global phonebook state
  const [book, setBook] = useState([]);
  // Controlled component for our form element
  const [newEntry, setEntry] = useState({ name: "", number: "" });
  console.count(`Rendering App component`);
  // Use either useState's lazy init function OR useEffect hook to avoid a infinite loop of setBook and axios network request
  // https://stackoverflow.com/questions/62050966/how-to-fetch-data-without-useeffect-hooks-in-react-function-component
  useEffect(() => {
    axios.get(`${base_url}/book`).then((response) => {
      setBook(response.data.phonebook);
    });
  }, []);
  console.log(book);
  const PhonebookEntry = ({ entry }) => {
    return (
      <li key="{entry.id}">
        Name : {entry.name} Number : {entry.number}
      </li>
    );
  };

  const addPhonebookEntry = (event) => {
    // Prevent the default action ie submitting the form, which we are going to do here anyway
    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/submit_event
    event.preventDefault();
    // event.target is the <Form /> we have defined the button submit on
    // console.log("add phonebook entry button clicked", event.target);
    setBook(book.concat(newEntry));
  };

  const findName = (e) => {
    let field = e.target.value;
    setSearchName(field);
    // console.log("Search name: ", searchName);
  };

  const onNameChange = (e) => {
    // e.target corresponds to the controlled <input> element
    setEntry({ ...newEntry, name: e.target.value });
  };

  const onNumChange = (e) => {
    // e.target corresponds to the controlled <input> element
    setEntry({ ...newEntry, number: e.target.value });
  };
  return (
    <div>
      <h1>Phonebook#69</h1>
      <label htmlFor="searchName">
        Search phonebook:
        <input
          id="searchName"
          type="text"
          value={searchName}
          onChange={findName}
        />
      </label>
      <ul>
        {/* Rendering a collection map() returns an array */}
        {/* key attribute added for outer elem PhonebookEntry in order to shut up react unique key props */}
        {searchName === ""
          ? book.map((each) => <PhonebookEntry key={each.id} entry={each} />)
          : book
              .filter((each) =>
                each.name.toLowerCase().startsWith(searchName.toLowerCase())
              )
              .map((each) => <PhonebookEntry key={each.id} entry={each} />)}
      </ul>
      <form onSubmit={addPhonebookEntry}>
        <label htmlFor="name ">
          Name:
          {/* A controlled component w/ just a value is rendered as a read-only field */}
          <input
            id="name"
            name="fullname"
            type="text"
            required
            value={newEntry.name}
            onChange={onNameChange}
          />
        </label>
        <br />
        <label htmlFor="number">
          Number:{" "}
          <input
            id="number"
            name="number"
            type="text"
            required
            value={newEntry.number}
            onChange={onNumChange}
          />
        </label>
        <br />
        <button type="submit">Save</button>
      </form>
    </div>
  );
}
