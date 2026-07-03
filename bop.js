};

plot();

const dragBox = {
  dragging: false,
  x1: 0,
  y1: 0,
  x2: 0,
  y2: 0,
};
canvas.addEventListener('pointerdown', e => {
  e.preventDefault();
  dragBox.dragging = true;
  dragBox.x1 = e.clientX;
  dragBox.y1 = e.clientY;
});
canvas.addEventListener('pointermove', e => {
  if (!dragBox.dragging) return;
  e.preventDefault();
  dragBox.x2 = e.clientX;
  dragBox.y2 = e.clientX;
});
window.addEventListener('pointerup', e => {
  if (!dragBox.dragging) return;
  e.preventDefault();
  dragBox.dragging = false;
  const { x1, x2, y1, y2 } = dragBox;
  const finalBox = {
    x: Math.min(x1, x2),
    y: Math.min(y1, y2),
    w: Math.abs(x2 - x1),
    h: Math.abs(x2 - x1),
  };
  // TODO: apply a transform to the canvas context, then redraw.
  plot();
});

const resetButton = document.getElementById('zoom-reset');
resetButton.addEventListener('click', () => {
  // TODO
});
