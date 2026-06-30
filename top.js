const canvas = document.querySelector("canvas");

const ctx = canvas.getContext("2d");

ctx.fillStyle = "black";
ctx.fillRect(0, 0, canvas.width, canvas.height);

ctx.lineWidth = 2;
ctx.strokeStyle = "lime";
