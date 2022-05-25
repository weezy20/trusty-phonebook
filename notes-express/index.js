const PORT = 3000;
const app = require("./app.js");
app.listen(PORT, () => {
  console.log(`Notes server running on localhost:${PORT}`);
});
