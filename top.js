const canvas = document.querySelector("canvas");

const ctx = canvas.getContext("2d");

ctx.fillStyle = "black";
ctx.fillRect(0, 0, canvas.width, canvas.height);
ctx.fillStyle = "#ffffff30";

ctx.lineWidth = 1;
ctx.strokeStyle = "transparent";

ctx.translate(0, -0.125 * canvas.height);
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
