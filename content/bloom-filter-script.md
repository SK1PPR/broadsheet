# THE BLOOM FILTER
### keylens · Vol. I, No. 1 — "Probably Yes, Definitely No"
Target runtime: ~8:00 · ~1,220 words @ 150 wpm
Beat names in `[brackets]` map to `m.mark()` calls in `examples/bloom_filter.rs`.

---

## §1 · THE HOOK — 0:00–0:55 `[hook]`

Somewhere on your machine right now, a database is about to do something expensive. A read request has arrived for a key — and that key might live in any one of a dozen files on disk. Opening each file to check would take milliseconds. Milliseconds, plural. In database time, that's an eternity.

`[hook-question]` So instead, the database asks a tiny structure in memory a strange question: "Have you seen this key before?" And the structure gives one of two answers. "Definitely not." Or — "probably."

`[hook-reveal]` That structure is a Bloom filter. It uses about one byte per item. It can be wrong — and that's not a bug. Being wrong, in exactly one direction, is the entire trick. Today: where it came from, how it works, how to tune it, and why half the infrastructure you use every day quietly depends on it.

## §2 · A PROBLEM FROM 1970 — 0:55–1:50 `[history]`

The year is 1970. Memory is measured in kilobytes and priced like real estate. Burton Howard Bloom publishes a five-page paper: *"Space/Time Trade-offs in Hash Coding with Allowable Errors."*

`[history-problem]` His example problem: automatic hyphenation. A dictionary of half a million words, where ninety percent follow simple rules, and ten percent need a disk lookup. Storing the whole dictionary in memory? Impossible. Bloom's insight: you don't need to store the words at all. You only need to answer *"could this word be one of the tricky ones?"* — and if you accept a small chance of a wasted disk lookup, the memory cost collapses.

`[history-insight]` Allow a little error, save a lot of space. In 1970 that bought you a computer that could hyphenate. Today the same five pages sit inside RocksDB, Cassandra, Chrome, and most CDNs. Not bad for a paper with one equation that fits on a napkin.

## §3 · HOW IT WORKS — 1:50–4:00 `[mechanics]`

Here's the whole machine: one array of bits, all starting at zero. Say, thirty-two of them. And a couple of hash functions — deterministic scramblers. Same input, same output, every time.

`[insert-1]` Let's insert the word "cat". We run "cat" through hash function one — it says position 5. Hash two says 11. Hash three says 26. Flip those three bits to one. That's it. That's an insert. We didn't store "cat". No letters, no pointers, no linked lists. Three ones in a sea of zeros.

`[insert-2]` Insert "dog": positions 3, 11 again — collisions are fine — and 20. Flip them. `[insert-3]` Insert "fish": 8, 14, 26. The array is filling up with overlapping fingerprints.

`[query-miss]` Now the payoff. Query: "is 'bird' in the set?" Hash "bird": positions 4, 14, 22. Check them. Position 14 is one... but position 4 is zero. Stop right there. If "bird" had ever been inserted, position 4 would be one. It isn't. So "bird" was **definitely never inserted**. No disk lookup, no full scan. One zero is proof of absence.

`[query-hit]` Query "cat": 5, 11, 26 — one, one, one. All set. Answer: "probably yes." Why only probably?

`[false-positive]` Here's the sting. Query "cow" — a word we never inserted. Hashes to 3, 8, 20. Position 3? Set — by "dog". Position 8? Set — by "fish". Position 20? Set — by "dog" again. All three bits are one, all lit by *other* words. The filter says "probably yes" — and it's wrong. That's a **false positive**: other members' fingerprints happen to cover yours.

`[asymmetry]` So the contract is asymmetric, and you should tattoo this on the inside of your eyelids: **"no" means no. "yes" means maybe.** A Bloom filter never misses a real member — bits are never turned off, so anything inserted keeps its bits lit. It only occasionally cries wolf. And notice what else it can't do: it can't list its members, and it can't delete — clearing "cat"'s bit at 11 would damage "dog", who shares it.

## §4 · TUNING THE MACHINE — 4:00–5:20 `[tuning]`

So how wrong is it? That's not fate — it's a dial you set. Three knobs: **m**, the number of bits. **n**, the number of items you'll insert. And **k**, the number of hash functions.

`[tuning-k]` Why multiple hash functions at all? One hash gives each item one fingerprint bit — and single bits collide constantly. More hashes make each item's fingerprint more specific: an impostor must now match *k* bits by luck, not one. But crank k too high and you light up the whole array — everything matches everything. There's a sweet spot, and it's known exactly: **k = (m/n) × ln 2**. About 0.7 bits-per-item ratio.

`[tuning-rule]` The rule of thumb every engineer actually memorizes: **ten bits per item, seven hash functions, one percent false positives.** A hundred million keys — say, every URL in a crawler's history — fits in 120 megabytes with a 1% error rate. Storing the URLs themselves would take gigabytes. That's the space-time bargain Bloom was selling in 1970, and the price hasn't gone up.

## §5 · THE UPGRADES — 5:20–6:35 `[variants]`

Fifty years of engineers have bolted on upgrades.

`[variant-counting]` Can't delete? The **counting Bloom filter** replaces each bit with a small counter. Insert increments, delete decrements, zero means clear. Cost: four times the memory. Nothing is free.

`[variant-blocked]` Cache misses hurt? A **blocked Bloom filter** confines each item's k bits to one 64-byte block — one cache line, one memory fetch instead of seven scattered ones. This is what RocksDB actually ships.

`[variant-multilevel]` Data arriving forever? **Scalable / multi-level Bloom filters** stack layers: when one fills to capacity, freeze it and start a new one on top, each layer tuned tighter than the last. Queries check layers newest-first — the same pyramid trick LSM trees use, and no accident: they live in the same codebases.

`[variant-cuckoo]` And the young challenger: the **cuckoo filter** — stores tiny fingerprints in a cuckoo hash table. Supports deletes, often beats Bloom on space below 3% error, at the cost of a slightly hairier insert. The 1970 original is still the default; the challenger wins on points in some weight classes.

## §6 · WHERE IT LIVES — 6:35–7:40 `[uses]`

Once you know the shape, you see it everywhere.

`[use-lsm]` **Storage engines** — RocksDB, LevelDB, Cassandra, HBase. Every on-disk file carries a Bloom filter of its keys. A point read asks the filter before touching disk; "definitely not" skips the file entirely. This is the single biggest read optimization in LSM-tree databases.

`[use-web]` **Browsers and CDNs.** Chrome historically used a Bloom filter to pre-screen URLs against its malware list — full check only on a filter hit. Akamai discovered three-quarters of requested objects are requested exactly once, so their caches only store an object the *second* time a filter has seen it — one-hit wonders never waste cache space.

`[use-misc]` **Databases** use them to skip partitions in joins. **Bitcoin's** light clients used them to subscribe to relevant transactions. **Medium** used them to avoid recommending articles you've already read. Anywhere the question is "can I skip the expensive thing?" — there's a Bloom filter answering it.

## §7 · THE CLOSE — 7:40–8:00 `[outro]`

A structure that stores nothing, sometimes lies, can't be listed, can't delete — and it's load-bearing infrastructure for the internet. The lesson generalizes: **perfect answers are expensive; calibrated doubt is nearly free.**

`[signoff]` That's this edition of keylens. Next issue: the skip list — what happens when you flip coins to build a search tree. Definitely subscribe. Probably.

---
*Production notes: record VO naturally, then `whisper --word_timestamps` → align sentence starts to the bracketed beat names via beats.json. Animation beats in examples/bloom_filter.rs use identical mark names.*
