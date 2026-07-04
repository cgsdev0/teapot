const canvas = document.querySelector("canvas");

const ctx = canvas.getContext("2d");

const clear = () => {
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#ffffff30";
};

clear();

ctx.lineWidth = 1;
ctx.strokeStyle = "transparent";

// we don't need this.
// we have canvas transforms at home
//
// ctx.translate(0, -0.125 * canvas.height);
// ctx.scale(-0.1, 0.1);
// ctx.translate(-canvas.width * 10, 0);

// canvas transforms at home:
let box = {
  x1: 0,
  y1: 0,
  x2: canvas.width,
  y2: canvas.height,
};

let pending_box = {
  ...box,
};

let zx = (x) => (x - box.x1) * (canvas.width / (box.x2 - box.x1));
let zy = (y) => (y - box.y1) * (canvas.height / (box.y2 - box.y1));

let ux = (x) => (x / canvas.width) * (box.x2 - box.x1) + box.x1;
let uy = (y) => (y / canvas.height) * (box.y2 - box.y1) + box.y1;
// zx = (x) => x;
// zy = (y) => y;

console.log(box);

let selecting = false;
canvas.addEventListener("pointerdown", (e) => {
  const { offsetX: x, offsetY: y } = e;
  pending_box.x1 = x;
  pending_box.y1 = y;
  selecting = true;
});

async function pointerup(e) {
  selecting = false;
  console.log(pending_box);

  const { x1, y1, x2, y2 } = pending_box;
  box = {
    x1: ux(Math.min(x1, x2)),
    y1: uy(Math.min(y1, y2)),
    x2: ux(Math.max(x1, x2)),
    y2: uy(Math.max(y1, y2)),
  };

  clear();
  plot();
}

window.addEventListener("pointerup", pointerup);
canvas.addEventListener("pointermove", (e) => {
  if (!selecting) return;
  const { offsetX: x, offsetY: y } = e;
  pending_box.x2 = x;
  pending_box.y2 = y;
  clear();
  plot();
  ctx.strokeStyle = "magenta";
  ctx.strokeRect(
    pending_box.x1,
    pending_box.y1,
    pending_box.x2 - pending_box.x1,
    pending_box.y2 - pending_box.y1,
  );
});

const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
