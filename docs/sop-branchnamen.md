# SOP: Branchnamen

Jeder Branch trägt die MOCO-Projektnummer am Ende. Ausnahme: `scratch`.

```
feat-import-button-p26059
└─┬─┘ └─────┬─────┘ └─┬──┘
prefix   thema     projektnummer
```

## Regeln

- **Prefix**: `feat`, `chore`, `fix`, `scratch`. Nichts anderes.
- **Zeichensatz**: nur `a-z`, `0-9` und `-`. Keine Slashes, keine
  Großbuchstaben, keine Umlaute — `ä`→`ae`, `ö`→`oe`, `ü`→`ue`, `ß`→`ss`.
- **Projektnummer**: immer am Ende, mit einem Bindestrich abgetrennt,
  kleingeschrieben. `P26059` → `p26059`. Am Ende, damit Tab-Autocomplete auf
  Prefix und Thema weiter funktioniert.
- **Verzeichnisname** = Branchname.
- **Retainer**: die Nummer des Retainers, nicht die des Einzelthemas —
  SONAX Development `p26002`, SONAX Website Maintenance `p26040`,
  VGH Social Recruiter Support `p26014`.

## Richtig

```
feat-import-button-p26059
chore-shopify-theme-p26054
fix-cookie-banner-p26043
scratch-cache-poc
scratch-cache-poc-p26059
```

## Falsch

```
feat/import-button-p26059      Slash
feat-import-button-P26059      Großbuchstaben
p26059-feat-import-button      Nummer nicht am Ende
feature-import-button-p26059   Prefix gibt es nicht
feat-größe-anpassen-p26040     Umlaut → feat-groesse-anpassen-p26040
```

## Scratch-Branches

Experimente und Wegwerf-Branches bekommen das Prefix `scratch`. Die
Projektnummer ist optional: anhängen, wenn das Experiment zu einem Kunden
gehört, sonst weglassen.

## Branches über `wt` anlegen

Nicht von Hand. `wt` erzeugt Branch, Worktree, Scaffold, Push und PR in einem
Schritt und kann den Namen nicht falsch bauen.

```
wt new              Wizard: Prefix → Thema → MOCO-Projekt → bestätigen
                    bei scratch: erste Zeile „no project“ = ohne Nummer
wt new cache poc    Worte werden nur bereinigt, kein Prefix, keine Nummer
wt rm               Worktree + lokalen Branch entfernen
```

`wt new` braucht `MOCO_API_KEY` in der Shell, sonst kann es die Projektliste
nicht holen. Ohne Key geht nur `scratch` ohne Nummer.

## Repo

[github.com/ole-treichel/dotfiles](https://github.com/ole-treichel/dotfiles),
Verzeichnis `wt/`. Installation: `./wt/install.sh`.

Details: [wt.md](wt.md) für die CLI, [wt-new-wizard.md](wt-new-wizard.md) für
den Wizard und die Entscheidungen dahinter.
