import { useState } from "react";
import Navigation from "./shared/Navigation";
import PageOne from "./pages/PageOne";
// import PageTwo from "./pages/PageTwo";
// import Footer from "./shared/Footer";

function Controller() {
  const [currentPage, setCurrentPage] = useState("page1");

  return (
    <div>
      <Navigation currentPage={currentPage} setCurrentPage={setCurrentPage} />
      <PageOne />
      {/* <Footer /> */}
    </div>
  );
}

export default Controller;
