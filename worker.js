import init, { setup, compile, load_font } from "/pkg/typst_renderer.js";

async function main() {
  await init();

  let ctx = setup();

  postMessage(true);

  addEventListener("message", (e) => {
    let [job, job_id] = e.data;
    if (job[0] == "render") {
      let [_method, source, px_per_pt, autosize, transparent] = job;

      console.log(source);
      let output = compile(ctx, source, px_per_pt, autosize, transparent);

      if (output.output === undefined) {
          let diag_text = "";
          for (let diag of output.diagnostics) {
              if (diag.severity === 1) {
                  diag_text += `Error at ${diag.range.start}..${diag.range.end}: ${diag.message}\n`;
              }
          }

          postMessage({ "diagnostics": diag_text, "job_id": job_id });
          return;
      }

      let blob = new Blob([output.output], { type: "image/png" });
      let url = URL.createObjectURL(blob);
      postMessage({ "url": url, "job_id": job_id });
    } else if (job[0] == "load_font") {
      let [_method, font_data] = job;

      let output = load_font(ctx, new Uint8Array(font_data));

      postMessage({ "result": output, "job_id": job_id });
    }
  });
}

main();
