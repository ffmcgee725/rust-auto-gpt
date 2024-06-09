import React, { useState } from "react";
import Logo from "./Logo";

type Props = {
  currentPage: string;
  setCurrentPage: any;
};

const Navigation: React.FC<Props> = ({ currentPage, setCurrentPage }) => {
  const [isOpen, setIsOpen] = useState(false);

  const pages = [
    { name: "Home", id: "home_page" },
    { name: "About", id: "about_page" },
  ];

  const toggleMenu = () => {
    setIsOpen(!isOpen);
  };

  const handleClick = (pageId: string) => {
    setCurrentPage(pageId);
    setIsOpen(false);
  };

  return (
    <header className="bg-white shadow-md">
      <div className="container mx-auto flex justify-between items-center px-4 py-2">
        <Logo />
        <div className="hidden sm:flex">
          {pages.map(({ name, id }) => (
            <button
              key={id}
              className={`ml-4 ${
                currentPage === id ? "text-blue-500" : "text-gray-700"
              }`}
              onClick={() => handleClick(id)}
            >
              {name}
            </button>
          ))}
        </div>
        <div className="sm:hidden">
          <button
            onClick={toggleMenu}
            className="block text-gray-500 hover:text-blue-500 focus:text-blue-500 focus:outline-none"
          >
            <svg viewBox="0 0 20 20" fill="currentColor" className="w-6 h-6">
              <path
                fillRule="evenodd"
                d="M2 3a1 1 0 011-1h14a1 1 0 010 2H3a1 1 0 01-1-1zm1 8a1 1 0 100-2h14a1 1 0 100-2H3a1 1 0 000 2zm0 5a1 1 0 100-2h14a1 1 0 100-2H3a1 1 0 000 2z"
                clipRule="evenodd"
              ></path>
            </svg>
          </button>
        </div>
      </div>
      {isOpen && (
        <div className="sm:hidden bg-white shadow-md">
          <nav>
            {pages.map(({ name, id }) => (
              <button
                key={id}
                className={`block w-full p-4 ${
                  currentPage === id ? "text-blue-500" : "text-gray-700"
                }`}
                onClick={() => handleClick(id)}
              >
                {name}
              </button>
            ))}
          </nav>
        </div>
      )}
    </header>
  );
};

export default Navigation;
