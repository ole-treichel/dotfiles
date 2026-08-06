use std::collections::BTreeSet;
use std::io::{stdout, IsTerminal};

use anyhow::{bail, Result};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

/// Full-screen list picker. Returns the chosen indices into `items`; an empty
/// result means the user cancelled.
pub fn pick(title: &str, items: &[String], multi: bool) -> Result<Vec<usize>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    if !stdout().is_terminal() {
        bail!("no terminal to open the picker in — pass the argument explicitly");
    }

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
            let [head, body, foot] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .areas(frame.area());

            let header = Line::from(vec![
                Span::styled(
                    format!(" {title} "),
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
                ),
                Span::raw(format!("{}  ", query)),
            ]);
            frame.render_widget(Paragraph::new(header), head);

            let rows: Vec<ListItem> = visible
                .iter()
                .map(|&i| {
                    let mark = if !multi {
                        String::new()
                    } else if selected.contains(&i) {
                        "[x] ".into()
                    } else {
                        "[ ] ".into()
                    };
                    ListItem::new(format!("{mark}{}", items[i]))
                })
                .collect();
            let list = List::new(rows)
                .highlight_symbol("> ")
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            frame.render_stateful_widget(list, body, &mut state);

            let help = if multi {
                "tab toggle · enter confirm · esc cancel · type to filter"
            } else {
                "enter select · esc cancel · type to filter"
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!(" {help}"),
                    Style::default().fg(Color::DarkGray),
                ))),
                foot,
            );
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
            KeyCode::Down => cursor = (cursor + 1).min(visible.len().saturating_sub(1)),
            KeyCode::Char('n') if ctrl => {
                cursor = (cursor + 1).min(visible.len().saturating_sub(1));
            }
            KeyCode::Up => cursor = cursor.saturating_sub(1),
            KeyCode::Char('p') if ctrl => cursor = cursor.saturating_sub(1),
            KeyCode::Tab if multi => {
                if let Some(&i) = visible.get(cursor) {
                    if !selected.remove(&i) {
                        selected.insert(i);
                    }
                }
                cursor = (cursor + 1).min(visible.len().saturating_sub(1));
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

fn filter(items: &[String], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }
    let needle = query.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, s)| s.to_lowercase().contains(&needle))
        .map(|(i, _)| i)
        .collect()
}
