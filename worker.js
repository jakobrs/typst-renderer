import init, { setup, compile_png, compile_svg, load_font } from "/pkg/typst_renderer.js";
// import init, { setup, compile, load_font } from "https://typst.ud2.no/pkg/typst_renderer.js";

function collect_diagnostics(diagnostics) {
  let diag_text = "";
  for (let diag of diagnostics) {
      if (diag.severity === 1) {
          diag_text += `Error at ${diag.range.start}..${diag.range.end}: ${diag.message}\n`;
      }
  }
  return diag_text;
}

async function main() {
  await init();

  let ctx = setup();

  postMessage(true);

  addEventListener("message", (e) => {
    let [job, job_id] = e.data;
    if (job[0] == "render_png") {
      let [_method, source, autosize, transparent, px_per_pt] = job;

      let output = compile_png(ctx, source, autosize, transparent, px_per_pt);

      if (output.output === undefined) {
        let diag_text = collect_diagnostics(output.diagnostics);
        postMessage({ "diagnostics": diag_text, "job_id": job_id });
        return;
      }

      let blob = new Blob([output.output], { type: "image/png" });
      let url = URL.createObjectURL(blob);
      postMessage({ "url": url, "job_id": job_id });
    } else if (job[0] == "render_svg") {
      let [_method, source, autosize, transparent] = job;

      let output = compile_svg(ctx, source, autosize, transparent);

      if (output.output === undefined) {
          let diag_text = collect_diagnostics(output.diagnostics);
          postMessage({ "diagnostics": diag_text, "job_id": job_id });
          return;
      }

      let blob = new Blob([output.output], { type: "image/svg+xml" });
      let url = URL.createObjectURL(blob);
      postMessage({ "url": url, "job_id": job_id });
    } else if (job[0] == "load_font") {
      let [_method, font_data] = job;

      let output = load_font(ctx, new Uint8Array(font_data));

      if (output === undefined) {
          postMessage({ "success": false, "job_id": job_id });
      } else {
          postMessage({ "success": true, "font_family": output, "job_id": job_id });
      }
    }
  });
}

main();
