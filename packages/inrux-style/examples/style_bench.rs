//! Reproducible selector microbenchmark; excludes native layout, paint, GPU and application startup.
use inrux_style::{CompiledStyles, Style, Viewport, parse_css};
use std::{hint::black_box, time::Instant};
/// Compare indexed matching with exhaustive matching using identical compiled rule data.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nodes = 10_000usize;
    for rule_count in [100usize, 1_000, 10_000] {
        let css = (0..rule_count)
            .map(|i| format!(".item{i} {{ padding: {}px; }}\n", i % 16))
            .collect::<String>();
        let start = Instant::now();
        let rules = parse_css(&css)?;
        let sheet = CompiledStyles::new(rules.clone());
        let compile = start.elapsed();
        let classes: Vec<Vec<String>> = (0..nodes)
            .map(|i| vec![format!("item{}", i % rule_count)])
            .collect();
        let viewport = Viewport {
            width: 1200.0,
            height: 860.0,
        };
        let mut durations = [Vec::new(), Vec::new()];
        for _ in 0..7 {
            let start = Instant::now();
            for names in &classes {
                let mut style = Style::new();
                for rule in sheet.matching("Button", None, black_box(names), viewport) {
                    style.overlay(&rule.style);
                }
                black_box(style);
            }
            durations[0].push(start.elapsed());
            let start = Instant::now();
            for names in &classes {
                let mut style = Style::new();
                // Each fixture has equal specificity; source iteration is already cascade order.
                for rule in &rules {
                    if rule.selector.matches("Button", None, black_box(names))
                        && rule.media.matches(viewport)
                    {
                        style.overlay(&rule.style);
                    }
                }
                black_box(style);
            }
            durations[1].push(start.elapsed());
        }
        for samples in &mut durations {
            samples.sort();
        }
        println!(
            "rules={rule_count} nodes={nodes} compile_and_index_us={} indexed_median_us={} exhaustive_median_us={}",
            compile.as_micros(),
            durations[0][3].as_micros(),
            durations[1][3].as_micros()
        );
    }
    Ok(())
}
