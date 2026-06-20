import init, { setup, compile } from "https://typst.ud2.no/pkg/typst_renderer.js";

async function main() {
  await init();

  let ctx = setup();

  postMessage(true);

  addEventListener("message", (e) => {
    let [[source, px_per_pt, autosize, transparent], job_id] = e.data;

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
  });
}

main();
