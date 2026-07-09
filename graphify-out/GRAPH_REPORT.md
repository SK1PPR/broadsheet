# Graph Report - .  (2026-07-09)

## Corpus Check
- 17 files · ~20,557 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 210 nodes · 339 edges · 19 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]

## God Nodes (most connected - your core abstractions)
1. `SceneBuilder<'a>` - 32 edges
2. `ActBuilder` - 21 edges
3. `Movie` - 13 edges
4. `draw_entity()` - 10 edges
5. `main()` - 6 edges
6. `Scene` - 6 edges
7. `scene_with_dot()` - 6 edges
8. `main()` - 5 edges
9. `BeatMap` - 5 edges
10. `main()` - 5 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### Community 0 - "Community 0"
Cohesion: 0.16
Nodes (1): SceneBuilder<'a>

### Community 1 - "Community 1"
Cohesion: 0.12
Nodes (15): abs_track_interpolates_and_holds(), Clip, evaluation_is_order_independent(), get_prop(), Prop, rel_chains_from_previous_end_and_revert_restores(), scene_with_dot(), set_prop() (+7 more)

### Community 2 - "Community 2"
Cohesion: 0.18
Nodes (1): ActBuilder

### Community 3 - "Community 3"
Cohesion: 0.22
Nodes (13): bezier_pts(), circle_pts(), draw_entity(), draw_head(), draw_page_chrome(), draw_path(), draw_scene(), draw_stroke_path() (+5 more)

### Community 4 - "Community 4"
Cohesion: 0.22
Nodes (1): Movie

### Community 5 - "Community 5"
Cohesion: 0.18
Nodes (5): fullscreen_pressed(), Opts, parse_opts(), run_loop(), take_seek()

### Community 6 - "Community 6"
Cohesion: 0.21
Nodes (6): act(), build_clip(), Clip, flash(), track(), Verb

### Community 7 - "Community 7"
Cohesion: 0.28
Nodes (4): ffmpeg_available(), markers_json(), Recorder, Sink

### Community 8 - "Community 8"
Cohesion: 0.54
Nodes (7): bit(), bit_x(), caption(), insert(), main(), probe(), retract()

### Community 9 - "Community 9"
Cohesion: 0.29
Nodes (2): Scene, SceneBuilder

### Community 10 - "Community 10"
Cohesion: 0.29
Nodes (5): Align, Entity, FontKind, Shape, StrokeStyle

### Community 11 - "Community 11"
Cohesion: 0.48
Nodes (6): estimate_bpm(), estimate_offset(), load_audio(), main(), onset_envelope(), Extract a beat map from an audio file -> beats.json.  Usage: python3 beatmap.py

### Community 12 - "Community 12"
Cohesion: 0.57
Nodes (2): BeatMap, main()

### Community 13 - "Community 13"
Cohesion: 0.38
Nodes (4): row(), row_spans_inclusive(), tree(), tree_centres_parent_over_children()

### Community 14 - "Community 14"
Cohesion: 0.8
Nodes (4): edge(), main(), node(), put()

### Community 15 - "Community 15"
Cohesion: 0.5
Nodes (1): Fonts

### Community 16 - "Community 16"
Cohesion: 0.67
Nodes (1): Easing

### Community 17 - "Community 17"
Cohesion: 0.67
Nodes (0): 

### Community 18 - "Community 18"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **13 isolated node(s):** `Extract a beat map from an audio file -> beats.json.  Usage: python3 beatmap.py`, `SceneBuilder`, `Shape`, `Align`, `FontKind` (+8 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 18`** (2 nodes): `features_demo.rs`, `main()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ActBuilder` connect `Community 2` to `Community 6`?**
  _High betweenness centrality (0.016) - this node is a cross-community bridge._
- **What connects `Extract a beat map from an audio file -> beats.json.  Usage: python3 beatmap.py`, `SceneBuilder`, `Shape` to the rest of the system?**
  _13 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._