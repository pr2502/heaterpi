async function get(api) {
  const res = await fetch(`api/${api}`, {
    "Accept": "application/json",
  });
  return await res.json();
}

async function post(api, body) {
  const res = await fetch(`api/${api}`, {
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json",
    },
    method: "POST",
    body: JSON.stringify(body),
  });
  return await res.json();
};

// Register click events
document.getElementById("on").onclick = async () => {
  const data = await post("heater/enable", {state: "on"});
  console.log(data);
};
document.getElementById("off").onclick = async () => {
  const data = await post("heater/enable", {state: "off"});
  console.log(data);
};

// Refresh dynamic data every second
const target = document.getElementById("target");
const hdo = document.getElementById("hdo");
const heat = document.getElementById("heat");
setInterval(async () => {
  const cameraData = await get("camera");
  hdo.setAttribute("class", JSON.stringify(cameraData.yellow));
  heat.setAttribute("class", JSON.stringify(cameraData.green));
  const heaterData = await get("heater/state");
  target.setAttribute("class", JSON.stringify(heaterData.target));
}, 1000);
