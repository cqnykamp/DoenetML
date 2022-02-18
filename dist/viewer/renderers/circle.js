import React, {useContext, useEffect, useState, useRef} from "../../_snowpack/pkg/react.js";
import useDoenetRender from "./useDoenetRenderer.js";
import {BoardContext} from "./graph.js";
export default function Circle(props) {
  let {name, SVs, actions, sourceOfUpdate} = useDoenetRender(props);
  const board = useContext(BoardContext);
  const [circleJXG, setCircleJXG] = useState({});
  let dragged = useRef(false);
  let pointerAtDown = useRef(false);
  let centerAtDown = useRef(false);
  let radiusAtDown = useRef(false);
  let throughAnglesAtDown = useRef(false);
  let previousWithLabel = useRef(false);
  let lastCenterFromCore = useRef(null);
  lastCenterFromCore.current = SVs.numericalCenter;
  useEffect(() => {
    if (board) {
      if (!(Number.isFinite(SVs.numericalCenter[0]) && Number.isFinite(SVs.numericalCenter[1]) && SVs.numericalRadius > 0)) {
        return;
      }
      var jsxCircleAttributes = {
        name: SVs.label,
        visible: !SVs.hidden,
        withLabel: SVs.showLabel && SVs.label !== "",
        fixed: !SVs.draggable || SVs.fixed,
        layer: 10 * SVs.layer + 5,
        strokeColor: SVs.selectedStyle.lineColor,
        highlightStrokeColor: SVs.selectedStyle.lineColor,
        strokeWidth: SVs.selectedStyle.lineWidth,
        highlightStrokeWidth: SVs.selectedStyle.lineWidth,
        dash: styleToDash(SVs.selectedStyle.lineStyle)
      };
      let newCircleJXG = board.create("circle", [[...SVs.numericalCenter], SVs.numericalRadius], jsxCircleAttributes);
      newCircleJXG.on("drag", function(e) {
        dragged.current = true;
        let centerCoords = calculateCenterPosition(e);
        props.callAction({
          action: actions.moveCircle,
          args: {
            center: centerCoords,
            radius: radiusAtDown.current,
            throughAngles: throughAnglesAtDown.current,
            transient: true
          }
        });
        newCircleJXG.center.coords.setCoordinates(JXG.COORDS_BY_USER, [...lastCenterFromCore.current]);
      });
      newCircleJXG.on("up", function(e) {
        if (dragged.current) {
          props.callAction({
            action: actions.finalizeCirclePosition
          });
        }
      });
      newCircleJXG.on("down", function(e) {
        dragged.current = false;
        pointerAtDown.current = [e.x, e.y];
        centerAtDown.current = [...newCircleJXG.center.coords.scrCoords];
        radiusAtDown.current = newCircleJXG.radius;
        throughAnglesAtDown.current = [...SVs.throughAngles];
      });
      previousWithLabel.current = SVs.showLabel && SVs.label !== "";
      setCircleJXG(newCircleJXG);
    }
    return () => {
      if (Object.keys(circleJXG).length !== 0) {
        circleJXG.off("drag");
        circleJXG.off("down");
        circleJXG.off("up");
        board.removeObject(circleJXG);
        setCircleJXG({});
      }
    };
  }, []);
  function calculateCenterPosition(e) {
    var o = board.origin.scrCoords;
    let calculatedX = (centerAtDown.current[1] + e.x - pointerAtDown.current[0] - o[1]) / board.unitX;
    let calculatedY = (o[2] - (centerAtDown.current[2] + e.y - pointerAtDown.current[1])) / board.unitY;
    return [calculatedX, calculatedY];
  }
  if (!board) {
    return null;
  }
  if (Object.keys(circleJXG).length !== 0) {
    if (!(Number.isFinite(SVs.numericalCenter[0]) && Number.isFinite(SVs.numericalCenter[1]) && SVs.numericalRadius > 0)) {
      return null;
    }
    if (board.updateQuality === board.BOARD_QUALITY_LOW) {
      board.itemsRenderedLowQuality[name] = circleJXG;
    }
    let validCoords = SVs.numericalCenter.every((x) => Number.isFinite(x));
    circleJXG.center.coords.setCoordinates(JXG.COORDS_BY_USER, [...SVs.numericalCenter]);
    circleJXG.setRadius(SVs.numericalRadius);
    let visible = !SVs.hidden;
    if (validCoords) {
      circleJXG.visProp["visible"] = visible;
      circleJXG.visPropCalc["visible"] = visible;
    } else {
      circleJXG.visProp["visible"] = false;
      circleJXG.visPropCalc["visible"] = false;
    }
    circleJXG.name = SVs.label;
    let withlabel = SVs.showLabel && SVs.label !== "";
    if (withlabel != previousWithLabel.current) {
      circleJXG.setAttribute({withlabel});
      previousWithLabel.current = withlabel;
    }
    circleJXG.needsUpdate = true;
    circleJXG.update();
    if (circleJXG.hasLabel) {
      circleJXG.label.needsUpdate = true;
      circleJXG.label.update();
    }
    board.updateRenderer();
  }
  if (SVs.hidden) {
    return null;
  }
  return /* @__PURE__ */ React.createElement("a", {
    name
  });
}
function styleToDash(style) {
  if (style === "solid") {
    return 0;
  } else if (style === "dashed") {
    return 2;
  } else if (style === "dotted") {
    return 1;
  } else {
    return 0;
  }
}
