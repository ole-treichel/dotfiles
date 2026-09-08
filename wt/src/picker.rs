use std::collections::BTreeSet;
use std::io::{stdout, IsTerminal};

use anyhow::{bail, Result};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, Padding, Paragraph};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;

/// How a tag reads at a glance: something to be careful about, or just context.
#[derive(Clone, Copy)]
pub enum Tone {
    Warn,
    Muted,
}

#[derive(Clone)]
pub struct Tag {
    text: String,
    tone: Tone,
}

/// One row: a name, a dimmer detail column, and zero or more tags.
#[derive(Clone, Default)]
pub struct Item {
    primary: String,
    secondary: String,
    tags: Vec<Tag>,
}

impl Item {
    pub fn new(primary: impl Into<String>) -> Self {
        Item {
            primary: primary.into(),
            ..Default::default()
        }
    }

    pub fn secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary = text.into();
        self
    }

    pub fn tag(mut self, text: impl Into<String>, tone: Tone) -> Self {
        self.tags.push(Tag {
            text: text.into(),
            tone,
        });
        self
    }

    fn haystack(&self) -> String {
        format!("{} {}", self.primary, self.secondary).to_lowercase()
    }

    /// Width of the rendered row, given the shared name column width.
    fn row_width(&self, name_width: usize) -> usize {
        let tags: usize = self.tags.iter().map(|t| t.text.chars().count() + 2).sum();
        let secondary = if self.secondary.is_empty() {
            0
        } else {
            self.secondary.chars().count() + 2
        };
        name_width + secondary + tags
    }
}

/// One terminal takeover, held open across however many steps a caller needs.
/// Each step used to `init` and `restore` for itself, which left and re-entered
/// the alternate screen in between — a visible flash between wizard steps.
pub struct Session {
    terminal: ratatui::DefaultTerminal,
}

impl Session {
    pub fn open() -> Result<Session> {
        if !stdout().is_terminal() {
            bail!("no terminal to open the picker in — pass the argument explicitly");
        }
        Ok(Session {
            terminal: ratatui::init(),
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

/// Full-screen list picker. Returns the chosen indices into `items`; an empty
/// result means the user cancelled.
pub fn pick(title: &str, items: &[Item], multi: bool) -> Result<Vec<usize>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    Session::open()?.pick(title, items, multi)
}

impl Session {
    pub fn pick(&mut self, title: &str, items: &[Item], multi: bool) -> Result<Vec<usize>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let name_width = items
            .iter()
            .map(|i| i.primary.chars().count())
            .max()
            .unwrap_or(0);
        // borders + padding + cursor marker (+ checkbox)
        let gutter = if multi { 8 } else { 6 };
        let help = if multi {
            " tab select · enter confirm · esc cancel "
        } else {
            " enter confirm · esc cancel "
        };
        let panel_width = items
            .iter()
            .map(|i| i.row_width(name_width) + gutter)
            .max()
            .unwrap_or(0)
            .max(help.chars().count() + 4);

        let mut query = String::new();
        let mut selected: BTreeSet<usize> = BTreeSet::new();
        let mut cursor = 0usize;
        let mut state = ListState::default();

        let result = loop {
            let visible: Vec<usize> = filter(items, &query);
            if cursor >= visible.len() {
                cursor = visible.len().saturating_sub(1);
            }
            state.select((!visible.is_empty()).then_some(cursor));

            let draw = self.terminal.draw(|frame| {
                // borders + filter line + gap
                let area = panel(frame.area(), panel_width as u16, items.len() as u16 + 4);

                let counter = if multi && !selected.is_empty() {
                    format!(" {} selected  {}/{} ", selected.len(), visible.len(), items.len())
                } else {
                    format!(" {}/{} ", visible.len(), items.len())
                };
                let block = chrome(title, Some(counter), help);
                let inner = block.inner(area);
                frame.render_widget(block, area);

                let [search, _gap, body] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .areas(inner);

                frame.render_widget(Paragraph::new(input_line(&query, "type to filter")), search);

                if visible.is_empty() {
                    frame.render_widget(
                        Paragraph::new(Line::styled("  no matches", Style::new().fg(DIM))),
                        body,
                    );
                    return;
                }

                let rows: Vec<ListItem> = visible
                    .iter()
                    .enumerate()
                    .map(|(row, &i)| {
                        ListItem::new(render_row(
                            &items[i],
                            name_width,
                            row == cursor,
                            multi.then(|| selected.contains(&i)),
                        ))
                    })
                    .collect();
                frame.render_stateful_widget(List::new(rows), body, &mut state);
            });
            if let Err(e) = draw {
                break Err(e.into());
            }

            let ev = match event::read() {
                Ok(ev) => ev,
                Err(e) => break Err(e.into()),
            };
            let Event::Key(key) = ev else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let last = visible.len().saturating_sub(1);
            match key.code {
                KeyCode::Esc => break Ok(Vec::new()),
                KeyCode::Char('c') if ctrl => break Ok(Vec::new()),
                KeyCode::Enter => {
                    let chosen: Vec<usize> = if multi && !selected.is_empty() {
                        selected.iter().copied().collect()
                    } else {
                        visible.get(cursor).copied().into_iter().collect()
                    };
                    break Ok(chosen);
                }
                KeyCode::Down => cursor = (cursor + 1).min(last),
                KeyCode::Char('n') if ctrl => cursor = (cursor + 1).min(last),
                KeyCode::Up => cursor = cursor.saturating_sub(1),
                KeyCode::Char('p') if ctrl => cursor = cursor.saturating_sub(1),
                KeyCode::Tab | KeyCode::BackTab if multi => {
                    if let Some(&i) = visible.get(cursor) {
                        if !selected.remove(&i) {
                            selected.insert(i);
                        }
                    }
                    cursor = (cursor + 1).min(last);
                }
                KeyCode::Backspace => {
                    query.pop();
                    cursor = 0;
                }
                KeyCode::Char(c) if !ctrl => {
                    query.push(c);
                    cursor = 0;
                }
                _ => {}
            }
        };
        result
    }

    /// One line of typed text in the same panel `pick` draws, so a wizard that
    /// mixes picking and typing keeps its shape from step to step. `None` means
    /// the user cancelled.
    pub fn prompt(&mut self, title: &str, hint: &str) -> Result<Option<String>> {
        let mut text = String::new();
        let result = loop {
            let draw = self
                .terminal
                .draw(|frame| render_prompt(frame, title, hint, &text));
            if let Err(e) = draw {
                break Err(e.into());
            }

            let ev = match event::read() {
                Ok(ev) => ev,
                Err(e) => break Err(e.into()),
            };
            let Event::Key(key) = ev else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Esc => break Ok(None),
                KeyCode::Char('c') if ctrl => break Ok(None),
                KeyCode::Enter => break Ok(Some(text.clone())),
                KeyCode::Backspace => {
                    text.pop();
                }
                KeyCode::Char(c) if !ctrl => text.push(c),
                _ => {}
            }
        };
        result
    }
}

/// Split out from the event loop so `TestBackend` can render it.
fn render_prompt(frame: &mut ratatui::Frame, title: &str, hint: &str, text: &str) {
    let help = " enter confirm · esc cancel ";
    // borders + padding + the ❯ marker and the cursor
    let width = (title.chars().count() + 8)
        .max(hint.chars().count() + 8)
        .max(help.chars().count() + 4);

    // borders + the one input line
    let area = panel(frame.area(), width as u16, 3);
    let block = chrome(title, None, help);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(input_line(text, hint)), inner);
}

/// The bordered panel every step draws itself into. `counter` is the right-hand
/// title the list uses and the text input has no use for.
fn chrome<'a>(title: &str, counter: Option<String>, help: &'a str) -> Block<'a> {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            format!(" {title} "),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Line::styled(help, Style::new().fg(DIM)).alignment(Alignment::Center));
    match counter {
        Some(counter) => {
            block.title_top(Line::styled(counter, Style::new().fg(DIM)).right_aligned())
        }
        None => block,
    }
}

/// `❯ what-you-typed▏`, with a dim hint while it is empty. Shared, so the
/// filter line and the text step are the same widget.
fn input_line(text: &str, hint: &str) -> Line<'static> {
    let mut spans = vec![
        Span::styled("❯ ", Style::new().fg(ACCENT)),
        Span::raw(text.to_string()),
        Span::styled("▏", Style::new().fg(ACCENT)),
    ];
    if text.is_empty() {
        spans.push(Span::styled(format!(" {hint}"), Style::new().fg(DIM)));
    }
    Line::from(spans)
}

/// `checked` is None for single-select, Some(state) when there are checkboxes.
fn render_row(item: &Item, name_width: usize, on_cursor: bool, checked: Option<bool>) -> Line<'_> {
    let mut spans = vec![Span::styled(
        if on_cursor { "❯ " } else { "  " },
        Style::new().fg(ACCENT),
    )];

    if let Some(checked) = checked {
        spans.push(if checked {
            Span::styled("✓ ", Style::new().fg(Color::Green))
        } else {
            Span::styled("· ", Style::new().fg(DIM))
        });
    }

    let name = format!("{:<name_width$}", item.primary);
    spans.push(if on_cursor {
        Span::styled(name, Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))
    } else {
        Span::raw(name)
    });

    if !item.secondary.is_empty() {
        spans.push(Span::styled(
            format!("  {}", item.secondary),
            Style::new().fg(DIM),
        ));
    }
    for tag in &item.tags {
        let style = match tag.tone {
            Tone::Warn => Style::new().fg(Color::Yellow),
            Tone::Muted => Style::new().fg(DIM).add_modifier(Modifier::ITALIC),
        };
        spans.push(Span::styled(format!("  {}", tag.text), style));
    }
    Line::from(spans)
}

/// A panel that grows with the list but never fills the whole terminal.
fn panel(area: Rect, want_w: u16, want_h: u16) -> Rect {
    let width = want_w.clamp(24, 100).min(area.width.saturating_sub(2));
    let height = want_h.clamp(3, 30).min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 3,
        width,
        height,
    }
}

/// Best match first. An empty query keeps the caller's order.
fn filter(items: &[Item], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }
    let needle = query.to_lowercase();
    let mut hits: Vec<(i32, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| score(&item.haystack(), &needle).map(|s| (s, i)))
        .collect();
    // sort_by_key is stable, so equal scores keep the caller's order.
    hits.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
    hits.into_iter().map(|(_, i)| i).collect()
}

/// Fuzzy subsequence match: every needle char has to appear in the haystack in
/// order, gaps allowed. `None` means no match.
///
/// Runs and word starts score far above a bare hit, so `amzbrand` ranks
/// `amazon brandstore redesign` over a row that merely happens to contain
/// those letters scattered about. The scan is greedy-leftmost rather than
/// optimal — with a few hundred rows the difference never shows.
fn score(haystack: &str, needle: &str) -> Option<i32> {
    let hay: Vec<char> = haystack.chars().collect();
    let mut total = 0i32;
    let mut from = 0usize;
    let mut prev_end = usize::MAX;
    for want in needle.chars() {
        let at = from + hay[from..].iter().position(|&c| c == want)?;
        total += 1;
        if at == prev_end {
            total += 8;
        }
        if at == 0 || !hay[at - 1].is_ascii_alphanumeric() {
            total += 6;
        }
        from = at + 1;
        prev_end = from;
    }
    // Tie-break towards the tighter row: same score on a shorter haystack is
    // the closer match.
    Some(total * 100 - hay.len() as i32)
}

#[cfg(test)]
mod tests {
    use super::{filter, score, Item};

    fn names(items: &[Item], query: &str) -> Vec<String> {
        filter(items, query)
            .into_iter()
            .map(|i| items[i].primary.clone())
            .collect()
    }

    #[test]
    fn matches_across_gaps() {
        assert!(score("amazon brandstore redesign", "amzbrand").is_some());
        assert!(score("amazon brandstore redesign", "brandstore").is_some());
        // Out of order is not a match.
        assert!(score("amazon brandstore redesign", "brandamz").is_none());
        assert!(score("amazon brandstore redesign", "zzz").is_none());
    }

    #[test]
    fn word_starts_outrank_scattered_letters() {
        let items = [
            Item::new("shopify theme").secondary("p26054 · sonax gmbh"),
            Item::new("social recruiter support").secondary("p26014 · vgh"),
        ];
        assert_eq!(
            names(&items, "sh"),
            ["shopify theme", "social recruiter support"]
        );
    }

    #[test]
    fn the_project_number_is_searchable() {
        let items = [
            Item::new("import-button").secondary("p26059 · vgh versicherungen"),
            Item::new("website maintenance").secondary("p26040 · sonax gmbh"),
        ];
        assert_eq!(names(&items, "p26059"), ["import-button"]);
        assert_eq!(names(&items, "26040"), ["website maintenance"]);
    }

    #[test]
    fn an_empty_query_keeps_the_callers_order() {
        let items = [Item::new("zeta"), Item::new("alpha")];
        assert_eq!(names(&items, ""), ["zeta", "alpha"]);
    }
}

#[cfg(test)]
mod render_tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// The panel as a plain string, one line per row, trailing blanks trimmed.
    pub fn prompt_frame(title: &str, hint: &str, text: &str) -> String {
        let mut terminal = Terminal::new(TestBackend::new(70, 9)).unwrap();
        terminal
            .draw(|frame| super::render_prompt(frame, title, hint, text))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let rows: Vec<String> = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect();
        rows.join("\n").trim_matches('\n').to_string()
    }

    #[test]
    fn the_text_step_wears_the_same_panel_as_the_list() {
        let drawn = prompt_frame(
            "feat-<description>-p…",
            "type the description",
            "import-button",
        );
        // Same rounded border, same accent title, same ❯ input line as `pick`.
        assert!(drawn.contains("╭"), "{drawn}");
        assert!(drawn.contains("╰"), "{drawn}");
        assert!(drawn.contains(" feat-<description>-p… "), "{drawn}");
        assert!(drawn.contains("❯ import-button▏"), "{drawn}");
        assert!(drawn.contains(" enter confirm · esc cancel "), "{drawn}");
        // Three rows of panel and nothing else: no stray lines above or below.
        assert_eq!(
            drawn.lines().filter(|l| !l.is_empty()).count(),
            3,
            "{drawn}"
        );
    }

    #[test]
    fn the_hint_shows_only_while_empty() {
        assert!(prompt_frame("t", "type the description", "").contains("type the description"));
        assert!(!prompt_frame("t", "type the description", "x").contains("type the description"));
    }
}
