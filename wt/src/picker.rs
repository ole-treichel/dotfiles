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

/// Full-screen list picker. Returns the chosen indices into `items`; an empty
/// result means the user cancelled.
pub fn pick(title: &str, items: &[Item], multi: bool) -> Result<Vec<usize>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    if !stdout().is_terminal() {
        bail!("no terminal to open the picker in — pass the argument explicitly");
    }

    let name_width = items
        .iter()
        .map(|i| i.primary.chars().count())
        .max()
        .unwrap_or(0);
    // borders + padding + cursor marker (+ checkbox)
    let chrome = if multi { 8 } else { 6 };
    let help = if multi {
        " tab select · enter confirm · esc cancel "
    } else {
        " enter confirm · esc cancel "
    };
    let panel_width = items
        .iter()
        .map(|i| i.row_width(name_width) + chrome)
        .max()
        .unwrap_or(0)
        .max(help.chars().count() + 4);

    let mut query = String::new();
    let mut selected: BTreeSet<usize> = BTreeSet::new();
    let mut cursor = 0usize;
    let mut state = ListState::default();

    let mut terminal = ratatui::init();
    let result = loop {
        let visible: Vec<usize> = filter(items, &query);
        if cursor >= visible.len() {
            cursor = visible.len().saturating_sub(1);
        }
        state.select((!visible.is_empty()).then_some(cursor));

        let draw = terminal.draw(|frame| {
            // borders + filter line + gap
            let area = panel(frame.area(), panel_width as u16, items.len() as u16 + 4);

            let counter = if multi && !selected.is_empty() {
                format!(" {} selected  {}/{} ", selected.len(), visible.len(), items.len())
            } else {
                format!(" {}/{} ", visible.len(), items.len())
            };
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(DIM))
                .padding(Padding::horizontal(1))
                .title(Span::styled(
                    format!(" {title} "),
                    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
                ))
                .title_top(Line::styled(counter, Style::new().fg(DIM)).right_aligned())
                .title_bottom(
                    Line::styled(help, Style::new().fg(DIM)).alignment(Alignment::Center),
                );
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let [search, _gap, body] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(1)])
                    .areas(inner);

            let mut prompt = vec![
                Span::styled("❯ ", Style::new().fg(ACCENT)),
                Span::raw(query.clone()),
                Span::styled("▏", Style::new().fg(ACCENT)),
            ];
            if query.is_empty() {
                prompt.push(Span::styled(" type to filter", Style::new().fg(DIM)));
            }
            frame.render_widget(Paragraph::new(Line::from(prompt)), search);

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
    ratatui::restore();
    result
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
    let height = want_h.clamp(5, 30).min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 3,
        width,
        height,
    }
}

fn filter(items: &[Item], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }
    let needle = query.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.haystack().contains(&needle))
        .map(|(i, _)| i)
        .collect()
}
