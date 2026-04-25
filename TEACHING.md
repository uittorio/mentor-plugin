# Teaching Methodologies — Research Foundation

This document captures the evidence-based teaching methodologies behind the `mentor+flow` skill design.

---

## Knowledge Type Framework

Teaching method selection in `mentor+flow` is driven by two independent signals: SM-2 depth (prior knowledge) and knowledge type.

**Knowledge type** classifies what kind of understanding a topic requires — specifically, what the learner needs to produce to demonstrate it:

| Knowledge type | Definition | Production artifact |
|---|---|---|
| **Procedural** | Know-how: a skill demonstrated through doing | Code written in editor |
| **Structural** | Know-how at scale: understanding how parts relate in a system | Diagram, architecture description, design decision |
| **Declarative** | Know-what: facts, concepts, principles explained accurately | Explanation in own words |

These map to established terms in educational psychology (declarative/procedural knowledge from Anderson, 1983; structural/schematic knowledge from cognitive load theory).

The full method matrix:

| SM-2 depth | Procedural | Structural | Declarative |
|---|---|---|---|
| `full` (new) | Worked example | Guided design exercise | Direct explanation |
| `light` (familiar) | Faded example | Describe/critique a design | Socratic |
| `skip` (mastery) | Retrieval practice | Trade-off analysis | Elaborative interrogation |

---

## Worked Examples → Faded Examples (Cognitive Load Theory)

**Researcher:** John Sweller  
**Key finding:** For novices, problem-solving too early overloads working memory. Instruction must first supply the schemas novices lack. Worked examples are the most effective technique for novice skill acquisition.

**Fading:** Steps are progressively removed from worked examples until the learner solves the problem independently. This is the research-backed bridge from "show me" to independent production.

**When to apply:** Procedural topics where SM-2 depth is `full` (worked example) or `light` (faded example).

Sources:
- [Worked-example effect — Wikipedia](https://en.wikipedia.org/wiki/Worked-example_effect)
- [The Guidance Fading Effect — Sweller](https://cogscisci.wordpress.com/wp-content/uploads/2019/08/sweller-guidance-fading.pdf)
- [Effects of worked examples on learning solution steps and knowledge transfer](https://www.tandfonline.com/doi/full/10.1080/01443410.2023.2273762)

---

## Retrieval Practice (Testing Effect)

**Key finding:** Testing from memory produces medium-to-large effect sizes for long-term retention. Writing beats re-reading or re-studying every time. Attempting to retrieve — even before initial exposure — improves later learning.

**When to apply:** Procedural topics at SM-2 depth `skip` — ask the developer to produce from memory without scaffolding.

Sources:
- [Effects of retrieval practice on retention — ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0959475225001434)
- [The Use of Retrieval Practice in the Health Professions — PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC12292765/)

---

## Spaced Repetition (Distributed Practice)

**Key finding:** Studying material over spaced intervals significantly improves long-term retention compared to massed practice ("cramming"), even with the same total study time. Already the foundation of the SM-2 knowledge tracking system in this plugin.

Sources:
- [Spaced Repetition and Retrieval Practice — Zeus Press](https://journals.zeuspress.org/index.php/IJASSR/article/view/425)
- [The Distributed Practice Effect — PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC12189222/)

---

## Socratic Method

**Key finding:** Effective for developing critical thinking *when the learner already has foundational knowledge*. Research shows only 1/3 to 1/2 of learners benefit from it alone — it must be combined with direct instruction for novices.

**Implication:** Socratic is the wrong default for new topics. It is appropriate only when the learner already has a schema to reason from.

**When to apply:** Declarative topics at SM-2 depth `light`. Not for `full` depth (no schema to reason from), and not for procedural topics at any depth (faded examples are more effective there).

Sources:
- [The Fact of Ignorance: Revisiting the Socratic Method — PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC4174386/)
- [Effectiveness of the Socratic Method — University of South Carolina](https://scholarcommons.sc.edu/cgi/viewcontent.cgi?article=1254&context=senior_theses)

---

## Elaborative Interrogation

**Key finding:** Asking "why does this work?" after a learner produces something deepens understanding and improves transfer. Cheaper than full Socratic dialogue — one targeted question rather than a chain.

**When to apply:** Declarative topics at SM-2 depth `skip`. Also useful as a follow-up move after any correct production to consolidate understanding.

Sources:
- [6 Evidence-Based Instructional Practices — Edutopia](https://www.edutopia.org/article/utilizing-evidence-based-instructional-practices/)
- [Teaching the science of learning — Springer](https://link.springer.com/article/10.1186/s41235-017-0087-y)

---

## Structural Methods (Design-Based Learning)

The three methods for structural knowledge — guided design exercise, describe/critique, trade-off analysis — are grounded in problem-based learning and case-based reasoning research.

**Guided design exercise** (`full`): Learners who are new to a system or architecture cannot reason from first principles without some scaffolding. Guided exercises that ask one design decision at a time reduce cognitive load while building the structural schema incrementally. Related to worked examples but applied to design rather than code.

**Describe or critique** (`light`): Asking learners to sketch or critique an existing design activates prior knowledge and surfaces misconceptions. More effective than asking for a blank-slate design, which can overwhelm working memory.

**Trade-off analysis** (`skip`): Learners with mastery benefit most from being pushed to the edges — when does this pattern break? What does it cost? This maps to the "evaluate" and "analyse" levels of Bloom's Taxonomy, which require deeper processing than simple recall or application.

**When to apply:** Structural topics (those requiring design of or reasoning about multiple components) at the corresponding SM-2 depth.

Sources:
- [Problem-Based Learning — Wikipedia](https://en.wikipedia.org/wiki/Problem-based_learning)
- [Bloom's Taxonomy — Vanderbilt CFT](https://cft.vanderbilt.edu/guides-sub-pages/blooms-taxonomy/)
- [Cognitive Load Theory and Instructional Design — University of Kentucky](https://www.uky.edu/~gmswan3/544/Cognitive_Load_&_ID.pdf)
