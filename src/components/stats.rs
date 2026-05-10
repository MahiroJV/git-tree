use crate::git::parser::RepoTree;
use chrono::{Datelike, Duration, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;

const STATS_CSS: &str = include_str!("../../assets/css/stats.css");

// ── Internal aggregate ────────────────────────────────────────────────────────

#[derive(Clone)]
struct ContributorStat {
    name: String,
    color: String,
    commit_count: usize,
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StatsScreen(tree: RepoTree, on_back: EventHandler<()>) -> Element {
    // ── Aggregate per-author stats ────────────────────────────────────────
    let mut map: HashMap<String, ContributorStat> = HashMap::new();
    for commit in &tree.commits {
        let entry = map
            .entry(commit.author_email.clone())
            .or_insert_with(|| ContributorStat {
                name: commit.author_name.clone(),
                color: commit.color.clone(),
                commit_count: 0,
            });
        entry.commit_count += 1;
    }
    let mut contributors: Vec<(String, ContributorStat)> = map.into_iter().collect();
    contributors.sort_by_key(|b| std::cmp::Reverse(b.1.commit_count));
    //contributors.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count));

    let total_commits = tree.commits.len();
    let max_count = contributors.first().map(|c| c.1.commit_count).unwrap_or(1);

    // ── Build day → commit-count map for heatmap ──────────────────────────
    let mut day_counts: HashMap<String, usize> = HashMap::new();
    for commit in &tree.commits {
        let key = commit.timestamp.format("%Y-%m-%d").to_string();
        *day_counts.entry(key).or_insert(0) += 1;
    }

    let max_day_count = day_counts.values().copied().max().unwrap_or(1);
    let total_active_days = day_counts.len();

    // ── Heatmap geometry ──────────────────────────────────────────────────
    // 52 columns (weeks), 7 rows (Mon–Sun)
    let cell: f64 = 12.0;
    let gap: f64 = 2.0;
    let stride = cell + gap;
    let left: f64 = 22.0; // room for single-char day label
    let top: f64 = 18.0; // room for month label
    let svg_w = left + 52.0 * stride + 8.0;
    let svg_h = top + 7.0 * stride + 2.0;

    // Align start to the Monday of 52 weeks ago
    let today = Utc::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday() as i64;
    let last_monday = today - Duration::days(days_since_monday);
    let first_monday = last_monday - Duration::weeks(51);

    // Build week columns and collect month label positions
    let mut weeks: Vec<Vec<(String, usize)>> = Vec::new(); // (date_key, count)
    let mut month_labels: Vec<(f64, String)> = Vec::new(); // (x, "Jan")
    let mut last_month: Option<u32> = None;

    for w in 0..52_usize {
        let week_start = first_monday + Duration::weeks(w as i64);

        // Emit a month label when the month rolls over
        let month = week_start.month();
        if Some(month) != last_month {
            let x = left + w as f64 * stride;
            month_labels.push((x, week_start.format("%b").to_string()));
            last_month = Some(month);
        }

        // 7 days per week
        let mut week_days: Vec<(String, usize)> = Vec::with_capacity(7);
        for d in 0..7_i64 {
            let date = week_start + Duration::days(d);
            let key = date.format("%Y-%m-%d").to_string();
            let count = if date <= today {
                *day_counts.get(&key).unwrap_or(&0)
            } else {
                0 // future cells are always empty
            };
            week_days.push((key, count));
        }
        weeks.push(week_days);
    }

    let repo_name = tree.repo_name.clone();
    let truncated = tree.truncated;

    rsx! {
        style { "{STATS_CSS}" }

        div {
            class: "stats-screen",

            // ── Header ────────────────────────────────────────────────────
            div { class: "stats-header",
                button {
                    class: "back-btn",
                    onclick: move |_| on_back.call(()),
                    "← BACK"
                }
                h2 { class: "settings-title", "// REPO STATS — {repo_name}" }
            }

            // ── Summary cards ─────────────────────────────────────────────
            div { class: "stats-summary",
                div { class: "stats-summary-card",
                    span { class: "stats-card-value", "{total_commits}" }
                    span { class: "stats-card-label", "COMMITS" }
                }
                div { class: "stats-summary-card",
                    span { class: "stats-card-value", "{contributors.len()}" }
                    span { class: "stats-card-label", "CONTRIBUTORS" }
                }
                div { class: "stats-summary-card",
                    span { class: "stats-card-value", "{total_active_days}" }
                    span { class: "stats-card-label", "ACTIVE DAYS" }
                }
                div { class: "stats-summary-card",
                    span { class: "stats-card-value", "{max_day_count}" }
                    span { class: "stats-card-label", "PEAK DAY" }
                }
                if truncated {
                    div { class: "stats-summary-card stats-summary-card--warn",
                        span { class: "stats-card-value", "500+" }
                        span { class: "stats-card-label", "CAPPED (first 500)" }
                    }
                }
            }

            // ── Commit heatmap ────────────────────────────────────────────
            div { class: "stats-section",
                h3 { class: "section-title", "COMMIT HEATMAP — LAST 52 WEEKS" }

                div { class: "heatmap-wrap",
                    svg {
                        width: "{svg_w}",
                        height: "{svg_h}",
                        xmlns: "http://www.w3.org/2000/svg",

                        // Day labels — M / W / F (odd rows would crowd the space)
                        text {
                            x: "0", y: "{top + 1.0 * stride + cell * 0.78}",
                            font_size: "8", fill: "var(--text-muted)",
                            font_family: "Space Mono, monospace",
                            "M"
                        }
                        text {
                            x: "0", y: "{top + 3.0 * stride + cell * 0.78}",
                            font_size: "8", fill: "var(--text-muted)",
                            font_family: "Space Mono, monospace",
                            "W"
                        }
                        text {
                            x: "0", y: "{top + 5.0 * stride + cell * 0.78}",
                            font_size: "8", fill: "var(--text-muted)",
                            font_family: "Space Mono, monospace",
                            "F"
                        }

                        // Month labels along the top
                        for (x_pos, month_name) in month_labels.iter() {
                            text {
                                key: "{month_name}-{x_pos}",
                                x: "{x_pos}",
                                y: "11",
                                font_size: "8",
                                fill: "var(--text-muted)",
                                font_family: "Space Mono, monospace",
                                "{month_name}"
                            }
                        }

                        // Heatmap cells — one rect per day
                        for (wi, week) in weeks.iter().enumerate() {
                            for (di, (date_key, count)) in week.iter().enumerate() {
                                {
                                    let cx = left + wi as f64 * stride;
                                    let cy = top + di as f64 * stride;

                                    // Choose fill and opacity by intensity bucket
                                    let (fill_color, cell_opacity): (&str, &str) = if *count == 0 {
                                        ("var(--bg-secondary)", "1")
                                    } else {
                                        let ratio = *count as f64 / max_day_count as f64;
                                        let op = if ratio > 0.75 {
                                            "1"
                                        } else if ratio > 0.50 {
                                            "0.75"
                                        } else if ratio > 0.25 {
                                            "0.50"
                                        } else {
                                            "0.30"
                                        };
                                        ("var(--accent)", op)
                                    };

                                    rsx! {
                                        rect {
                                            key: "{date_key}",
                                            x: "{cx}",
                                            y: "{cy}",
                                            width: "{cell}",
                                            height: "{cell}",
                                            rx: "2",
                                            fill: "{fill_color}",
                                            opacity: "{cell_opacity}",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Intensity legend
                div { class: "heatmap-legend",
                    span { "LESS" }
                    for (i, op) in ["0.15", "0.30", "0.50", "0.75", "1.0"].iter().enumerate() {
                        svg {
                            key: "{i}",
                            width: "12", height: "12",
                            xmlns: "http://www.w3.org/2000/svg",
                            rect {
                                width: "12", height: "12", rx: "2",
                                fill: "var(--accent)",
                                opacity: "{op}",
                            }
                        }
                    }
                    span { "MORE" }
                }
            }

            // ── Contributor leaderboard ───────────────────────────────────
            div { class: "stats-section",
                h3 { class: "section-title", "CONTRIBUTOR LEADERBOARD" }
                div { class: "leaderboard",
                    for (rank, (email, stat)) in contributors.iter().enumerate() {
                        {
                            let bar_w = (stat.commit_count as f64
                                / max_count as f64 * 100.0) as u32;
                            let share = format!(
                                "{:.1}%",
                                stat.commit_count as f64 / total_commits as f64 * 100.0
                            );
                            let color = stat.color.clone();
                            let rank_num = rank + 1;

                            rsx! {
                                div {
                                    class: "leaderboard-row",
                                    key: "{email}",

                                    span { class: "lb-rank", "#{rank_num}" }
                                    span { class: "lb-dot", style: "background: {color};" }

                                    div { class: "lb-info",
                                        span { class: "lb-name",  "{stat.name}" }
                                        span { class: "lb-email", "{email}" }
                                    }

                                    div { class: "lb-bar-wrap",
                                        div {
                                            class: "lb-bar",
                                            style: "width: {bar_w}%; background: {color};"
                                        }
                                    }

                                    span { class: "lb-count", "{stat.commit_count}" }
                                    span { class: "lb-share", "{share}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
