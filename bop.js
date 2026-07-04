let idx = 0;

if (bounding_boxes && bounding_boxes.length) {
  box = bounding_boxes[idx];
  console.log("FACE: ", box.face);
  plot();
  draw_triangle(box);

  window.addEventListener("keydown", (e) => {
    if (e.code === "AltLeft") {
      box = original_box;
      clear();
      plot();
      draw_box(bounding_boxes[idx]);
      draw_triangle(box);
    } else if (e.code === "Enter") {
      idx += 1;
      box = bounding_boxes[idx];
      console.log("FACE: ", box.face);
      clear();
      plot();
      draw_triangle(box);
    }
  });

  window.addEventListener("keyup", (e) => {
    if (e.code === "AltLeft") {
      box = bounding_boxes[idx];
      clear();
      plot();
      draw_triangle(box);
    }
  });
} else {
  plot();
}
