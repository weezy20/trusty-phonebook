import React from "react";
// import ReactDOM from 'react-dom'
import { useEffect, useState } from "react";
import axios from "axios";
const base_url = "http://localhost";

export default function App() {
  const [book, setBook] = useState([]);
  console.log(`Connecting to ${base_url}`);
  axios.get(`${base_url}/book`).then((response) => {
    setBook(response.data.phonebook);
    console.log(book);
  });

  return (
    <div>
      <h1>Phonebook#69</h1>
      <ul>
        {book.map((each) => (<PhonebookEntry entry={each} />))}
      </ul>
    </div>
  );
}

const PhonebookEntry = ( props ) => {
  return (
    <li key={props.entry.id}>
      Name : {props.entry.name} Number : {props.entry.number}
    </li>
  );
};
