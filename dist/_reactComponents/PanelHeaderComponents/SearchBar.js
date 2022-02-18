import React, {useState} from "../../_snowpack/pkg/react.js";
import {doenetComponentForegroundInactive, doenetComponentForegroundActive} from "./theme.js";
import {FontAwesomeIcon} from "../../_snowpack/pkg/@fortawesome/react-fontawesome.js";
import {faSearch, faTimes} from "../../_snowpack/pkg/@fortawesome/free-solid-svg-icons.js";
export default function Searchbar(props) {
  const [searchTerm, setSearchTerm] = useState("");
  const [cancelShown, setCancelShown] = useState("hidden");
  const [searchShown, setSearchShown] = useState("visible");
  const [labelVisible, setLabelVisible] = useState(props.label ? "static" : "none");
  const [align, setAlign] = useState(props.vertical ? "static" : "flex");
  const marginLeft = props.noSearchButton ? 80 : 26;
  var searchBar = {
    margin: "0px",
    height: "24px",
    border: `2px solid black`,
    borderRadius: "5px",
    position: "relative",
    padding: "0px 70px 0px 30px",
    color: "#000",
    overflow: "hidden",
    width: "220px",
    fontSize: "14px"
  };
  if (props.width) {
    if (props.width === "menu") {
      searchBar.width = "130px";
    }
  }
  let cancelLeftMargin = Number(searchBar.width.split("px")[0]) + marginLeft + "px";
  var cancelButton = {
    float: "right",
    margin: `6px 0px 0px ${cancelLeftMargin}`,
    position: "absolute",
    zIndex: "2",
    border: "0px",
    backgroundColor: "#FFF",
    visibility: `${cancelShown}`,
    color: "#000",
    overflow: "hidden",
    outline: "none"
  };
  var searchIcon = {
    margin: "6px 0px 0px 6px",
    position: "absolute",
    zIndex: "1",
    color: "#000",
    overflow: "hidden"
  };
  var submitButton = {
    position: "absolute",
    display: "inline",
    margin: "0px 0px 0px -60px",
    zIndex: "2",
    height: "28px",
    border: `2px solid black`,
    backgroundColor: `${doenetComponentForegroundActive}`,
    color: "#FFFFFF",
    borderRadius: "0px 5px 5px 0px",
    cursor: "pointer",
    fontSize: "12px",
    overflow: "hidden"
  };
  var disable = "";
  if (props.disabled) {
    submitButton.backgroundColor = "#e2e2e2";
    submitButton.color = "black";
    submitButton.cursor = "not-allowed";
    searchBar.cursor = "not-allowed";
    disable = "disabled";
  }
  var label = {
    value: "Label:",
    fontSize: "14px",
    display: `${labelVisible}`,
    margin: "0px 5px 2px 0px"
  };
  var container = {
    display: `${align}`,
    width: "235px",
    alignItems: "center"
  };
  var searchButton = /* @__PURE__ */ React.createElement("button", {
    style: submitButton,
    onClick: searchSubmitAction
  }, "Search");
  if (props.noSearchButton) {
    searchButton = "";
    console.log(cancelLeftMargin);
  }
  if (props.placeholder) {
    searchBar.placeholder = props.placeholder;
  }
  if (props.label) {
    label.value = props.label;
  }
  if (props.ariaLabel) {
    searchBar.ariaLabel = props.ariaLabel;
  }
  if (props.alert) {
    searchBar.border = "2px solid #C1292E";
  }
  function clearInput() {
    setSearchTerm("");
    setCancelShown("hidden");
    if (props.onChange) {
      props.onChange("");
    }
  }
  function onChange(e) {
    let val = e.target.value;
    setSearchTerm(val);
    if (val === "") {
      setCancelShown("hidden");
    } else {
      setCancelShown("visible");
    }
    if (props.onChange) {
      props.onChange(val);
    }
  }
  function handleBlur(e) {
    if (props.onBlur)
      props.onBlur(e);
  }
  function handleKeyDown(e) {
    if (props.onKeyDown)
      props.onKeyDown(e);
  }
  function searchSubmitAction() {
    if (props.onSubmit) {
      props.onSubmit(searchTerm);
    }
  }
  let autoFocus = false;
  if (props.autoFocus) {
    autoFocus = true;
  }
  return /* @__PURE__ */ React.createElement("div", {
    style: container
  }, /* @__PURE__ */ React.createElement("p", {
    style: label
  }, label?.value), /* @__PURE__ */ React.createElement("div", {
    style: {display: "table-cell"}
  }, /* @__PURE__ */ React.createElement(FontAwesomeIcon, {
    icon: faSearch,
    style: searchIcon
  }), /* @__PURE__ */ React.createElement("button", {
    style: cancelButton,
    onClick: () => {
      clearInput();
    }
  }, /* @__PURE__ */ React.createElement(FontAwesomeIcon, {
    icon: faTimes
  })), /* @__PURE__ */ React.createElement("input", {
    id: "search",
    type: "text",
    placeholder: props.placeholder ? searchBar.placeholder : "Search...",
    style: searchBar,
    onChange,
    onBlur: (e) => {
      handleBlur(e);
    },
    onKeyDownCapture: (e) => {
      handleKeyDown(e);
    },
    disabled: disable,
    value: searchTerm,
    onKeyDown: (e) => {
      if (e.key === "Enter") {
        searchSubmitAction();
      }
    },
    autoFocus,
    "aria-label": searchBar.ariaLabel
  }), /* @__PURE__ */ React.createElement("div", {
    style: {padding: "3px", display: "inline"}
  }), searchButton));
}
