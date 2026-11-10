# Tether: Scientific Rationale & Neurodivergent Research Foundation 🧠🔬

> **Document Version:** 1.3.0  
> **Author:** Kim Bech <kimbech@gmail.com> & The Tether Project  
> **Target Audience:** Developers, Cognitive Scientists, Neurodivergent Individuals (ADHD, Autism, Dyslexia, AuDHD), and Productivity Researchers  

---

## Executive Summary

Traditional productivity paradigms—most notably the 25-minute Pomodoro Technique, binary streak counters, rigid schedule blocks, and invasive task reminders—were developed under the implicit assumption of a **neurotypical executive control architecture**. For individuals across the neurodivergent spectrum—including **Attention-Deficit/Hyperactivity Disorder (ADHD)**, **Autism Spectrum Conditions (ASC)**, **Developmental Dyslexia**, and **AuDHD**—these conventional paradigms consistently trigger **task-initiation paralysis**, **traumatic context loss**, **autistic transition distress**, **phonological working memory overload**, and **shame-induced habit abandonment** (the *"What-the-Hell"* effect).

**Tether** is built from the ground up as a clinical-grade cognitive prosthetic (*cognitive orthotic*) for neurodivergent knowledge workers. Every structural decision in Tether—from its count-up Flowmodoro engine and 5-minute micro-start to its hybrid 75–90-second acoustic field recordings, 40 Hz gamma neural entrainment, RAM-dump impulse buffers, and externalized context anchors—is grounded in peer-reviewed neuroscience, cognitive ergonomics, and attention restoration theory.

This document outlines the scientific literature, neurobiological mechanisms, and empirical rationale underpinning each subsystem of Tether across ADHD, Autism, and Dyslexia.

---

## 1. Neurobiological Primer: The ADHD Executive Architecture

To understand why conventional software fails ADHD developers, one must examine the fronto-striatal and catecholaminergic differences that characterize the ADHD brain:

```
                  ┌────────────────────────────────────────┐
                  │ Prefrontal Cortex (PFC) Hypo-arousal   │
                  │ Reduced baseline tonic dopamine & NE   │
                  └──────────────────┬─────────────────────┘
                                     │
         ┌───────────────────────────┴───────────────────────────┐
         ▼                                                       ▼
┌─────────────────────────────────┐             ┌─────────────────────────────────┐
│     Working Memory Deficit      │             │    Cognitive Inertia / Decay    │
│ Rapid decay of mental context;  │             │ Extreme activation energy to    │
│ high task-switching penalty     │             │ start; deep hyperfocus once in  │
│ (Barkley 1997; Baddeley 2000)   │             │ motion (Brown 2005; Groen 2020) │
└─────────────────────────────────┘             └─────────────────────────────────┘
```

1. **The Fronto-Striatal Dopaminergic Deficit:**  
   Neuroimaging studies (Volkow et al., 2009; Castellanos et al., 2006) demonstrate that individuals with ADHD have significantly reduced dopamine $D_2/D_3$ receptor availability and dopamine transporter (DAT) binding in the striatum and prefrontal cortex. This causes **low tonic dopamine**, leading to chronically under-stimulated reward and salience networks.
2. **Working Memory & Executive Gating:**  
   As formulated in Russell Barkley's Unified Model of Executive Function (Barkley, 1997, 2012), ADHD is fundamentally an impairment of *self-regulation across time*, not an inability to pay attention. Working memory is fragile: when external stimuli pull attention away, internally rehearsed task representations evaporate rapidly (Martinussen et al., 2005; Kofler et al., 2011).
3. **Time Blindness & Temporal Discounting:**  
   Individuals with ADHD experience severe temporal discounting—future events and past intervals lack emotional salience until they become immediate crises (Barkley, 1997; Ptacek et al., 2019). Internal estimation of elapsed time is significantly less accurate, requiring continuous, low-friction external time representation.

---

## 2. Core Subsystems & Scientific Rationales

### 2.1 The Flowmodoro Engine: Count-Up Focus & Proportional Rest

#### The Problem with Fixed Pomodoros (25m / 5m)
The traditional Pomodoro Technique enforces a rigid countdown from 25:00. When the timer expires, an auditory alarm orders the user to cease working immediately. For knowledge workers and developers with ADHD:
- **Hyperfocus Interruption:** Entering hyperfocus—a state of heightened, intense task absorption (Groen et al., 2020; Hupfeld et al., 2019)—often requires 30 to 45 minutes of activation energy. An arbitrary alarm ringing at 25 minutes cuts the user off precisely as their code abstractions and variable graphs are coalescing.
- **Catastrophic Context Re-entry Cost:** Seminal research by Mark, Gudith, and Klocke (2008) at the University of California, Irvine, demonstrated that resuming interrupted cognitive work requires an average of **23 minutes and 15 seconds**. For an ADHD mind, whose working memory decay rate is heightened (Kofler et al., 2011), mid-thought interruptions can derail an entire afternoon.

#### The Tether Solution: Count-Up Stopwatch & Dynamic Rest Ratio
Tether counts **UP** from `00:00`:
- **Open-Ended Flow:** Users work uninterrupted for as long as their intrinsic focus and neurochemical momentum persist (Csíkszentmihályi, 1990).
- **Proportional Recovery (Ego Depletion Model):** According to the strength model of self-control (Baumeister et al., 1998; Sonnentag, 2001; Fritz et al., 2011), mental stamina depletes proportionally to the duration and intensity of sustained executive effort. Tether utilizes a customizable ratio (default **45:15**, or 3:1):
  $$\text{Earned Break} = \frac{\text{Focus Time}}{\text{Ratio Multiplier}}$$
  A 60-minute deep programming session earns 20 minutes of guilt-free recovery. A shorter 15-minute bout earns 5 minutes.
- **Banked Break Time:** When returning early from an earned break (e.g., returning after 8 minutes of a 15-minute break), traditional timers wipe the remaining time. Tether **banks** unspent rest and automatically resumes the focus counter, preserving cognitive flexibility.

---

### 2.2 The "Just 5 Minutes" Micro-Start: Overcoming Initiation Inertia

#### The Neurobiology of Task Initiation
The hardest obstacle for the ADHD brain is rarely *doing* the task—it is the **activation threshold** required to cross the threshold from rest to action (Brown, 2005; Pychyl & Flett, 2012). The brain perceives a monumental, nebulous programming project as a threat, triggering amygdala-driven avoidance and dopamine-seeking displacement behaviors.

#### The Tether Solution
- **Micro-Commitment & Lowering Activation Energy:** Rooted in cognitive behavioral therapy (CBT) behavioral activation (Beck, 1979) and implementation intentions (Gollwitzer & Sheeran, 2006), Tether's `5m Micro-Start` establishes a contract: *"Commit to typing for only 300 seconds. After 5 minutes, you have unconditional permission to stop."*
- **The Zeigarnik Effect:** First observed by Soviet psychologist Bluma Zeigarnik (1927), incomplete tasks produce an underlying cognitive tension that naturally compels the brain to finish what has been started. Once the developer opens their editor and starts typing code for 5 minutes, the task is no longer an abstract fear—it is an active, open cognitive loop.
- **Seamless Glide Transition:** When the 5-minute countdown finishes, Tether emits a gentle 528 Hz chime and **glides seamlessly into standard Flowmodoro counting up from 05:00**, allowing momentum to carry the session forward without requiring any user input.
- **Shame-Free Abort:** If the user stops before 5 minutes, Tether presents a reassuring message (*"Good attempt! Totally fine to stop here."*), deliberately short-circuiting Rejection Sensitive Dysphoria (RSD).

---

### 2.3 Hybrid Focus Soundscapes, Stochastic Resonance & 40 Hz Gamma Entrainment

```
Auditory Input:   [ White / Brown Noise Base ]
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
     Moderate Brain Arousal          40 Hz Gamma Entrainment
     Stochastic Resonance in PFC     Auditory Steady-State Response
     (Sikström & Söderlund 2007)     (Adaikkan & Tsai 2020)
               │                               │
               └───────────────┬───────────────┘
                               ▼
        Stabilized Executive Focus & Intrusive Masking
```

Tether features a hybrid audio architecture combining mathematically generated procedural noise with long-duration authentic nature field recordings.

#### 1. Stochastic Resonance & The Moderate Brain Arousal (MBA) Model
In neurotypical individuals, excessive background noise impairs cognitive performance. However, groundbreaking clinical research by Söderlund, Sikström, and colleagues (2007, 2010, 2016) revealed a paradoxical finding: **auditory noise significantly improves cognitive, memory, and executive performance in individuals with ADHD**.

This is explained by the **Moderate Brain Arousal (MBA) model**:
- Low tonic dopamine in ADHD leads to an under-aroused prefrontal cortex with poor signal-to-noise ratio in synaptic neurotransmission.
- When continuous auditory noise is introduced, it introduces **stochastic resonance**—a non-linear physical phenomenon where sub-threshold cognitive signals are amplified by adding random noise to the system.
- Neural firing thresholds in fronto-striatal circuits become easier to reach, stabilizing attention.

#### 2. Soundscape Profiles & Psychoacoustic Engineering
- **Brown Noise ($1/f^2$ Power Spectrum):** Brown (red) noise attenuates higher frequencies by 6 dB per octave. The deep, rumbling low-frequency profile provides acoustic shielding against office chatter and erratic environmental transients while effectively masking tinnitus, a condition with high comorbidity in neurodivergent populations (Zhou et al., 2012; Halin et al., 2014).
- **Pink Noise ($1/f$ Power Spectrum):** Pink noise exhibits equal energy per octave band. Its fractal power distribution mirrors electrical rhythms found throughout the human central nervous system and biological systems (Halley, 1996; Suzuki et al., 2020).
- **Green Noise (~500 Hz Nature Band):** Individuals with ADHD and autism frequently suffer from **misophonia** and sensory over-responsivity (Brout et al., 2018; Kumar et al., 2017). Green noise isolates the lush 200 Hz to 2,000 Hz mid-frequency band, aggressively filtering out piercing high-frequency sibilance (>3 kHz) and muddy sub-bass rumbles (<150 Hz).
- **40 Hz Gamma Pulse (Neural Entrainment):** Gamma-band oscillations (~40 Hz) in the cerebral cortex are the electrophysiological hallmark of attentional binding, working memory integration, and conscious sensory processing. Research pioneered by the MIT Tsai Laboratory (Iaccarino et al., 2016; Adaikkan & Tsai, 2020; Park et al., 2020) demonstrates that **40 Hz sensory stimulation** drives profound auditory steady-state responses (ASSR), entrains micro-synchrony in cortical circuits, and enhances cognitive processing speed. Tether modulates a warm brown noise carrier with an exact 40 Hz isochronic sinusoid ($1.0 + 0.32 \sin(2\pi \cdot 40 \cdot t)$), providing a steady rhythmic metronome for attention.

#### 3. Real Nature Sounds & Attention Restoration Theory (ART)
Rachel and Stephen Kaplan’s **Attention Restoration Theory (ART)** (Kaplan, 1995; Ulrich, 1984; Ratcliffe et al., 2013) posits that voluntary directed attention is a finite, exhaustible cognitive resource. Nature sounds (such as flowing water, gentle rain, and woodland wind) engage **"soft fascination"**—an effortless, involuntary attention that allows the executive directed-attention mechanisms in the prefrontal cortex to rest and replenish.

#### 4. The Psychoacoustics of Long Loops (75–90s) & Mismatch Negativity (MMN)
Why do short audio loops (10–20 seconds) fail for ADHD users?
- The human auditory cortex constantly constructs predictive models of the acoustic environment (Winkler, 2007).
- When a distinct audio transient (such as an identifiable water droplet splash or wind gust) repeats every 15 seconds, it triggers the brain's **Mismatch Negativity (MMN)** and P300 novelty-detection ERPs (Näätänen et al., 2007).
- For an ADHD listener with sensory gating vulnerabilities, the brain locks onto the repetition. Instead of focusing on code, the user involuntarily waits for the next anticipated repeat, shattering concentration.
- **Tether’s 75–90-Second Loops:** Tether utilizes master field recordings (up to 32 minutes long in raw format) crossfaded over 3–4 seconds into **75 to 90-second loops** (1.5 minutes). At this duration, the sequence of acoustic transients vastly exceeds working memory's temporal window (Baddeley, 2000), making the audio indistinguishable from an infinite, non-repeating natural environment.

---

### 2.4 Context Anchoring & RAM-Dump: Mitigating Working Memory Decay

#### Working Memory Amnesia & The Extended Mind
Clark and Chalmers (1998) introduced the **Extended Mind Thesis**: cognition does not end at the biological skull; external artifacts (notebooks, screens, digital files) actively constitute part of the cognitive system. In ADHD, where biological working memory is prone to immediate flushing upon distraction (Martinussen et al., 2005):
- Stepping away to grab lunch, attend a stand-up, or take an earned break often results in complete loss of mental register: *"What was I in the middle of? What file was I editing? What bug was I chasing?"*
- Rebuilding this context requires significant executive energy, often precipitating task abandonment.

#### The Tether Solution
- **Automated Context Anchor:** Tether records the user's active Git repository, branch, relative file path, line number, column, and last code note to `~/.local/state/tether/context_anchor.json`.
- **1-Click Editor Resumption:** The Flow tab provides an instant **"Gjenoppta i Neovim / VS Code 🚀"** button. One click switches workspaces, focuses the editor, and jumps the cursor directly to the exact file and line where the developer left off, eliminating re-entry friction entirely.
- **The 60-Second RAM-Dump (`[NÅ]`, `[STATUS]`, `[NESTE STEG]`):** Rooted in Sonuga-Barke and Castellanos' (2007) **Default Mode Network (DMN) interference hypothesis**, when working memory is saturated with competing thoughts, the brain struggles to suppress task-negative default network activity. Rapidly dumping mental clutter into an unformatted scratchpad frees working memory slots, restoring task-positive executive focus.

---

### 2.5 Interactive Impulse Parking & Distraction Shielding

#### Prospective Memory & Goal Shielding
Prospective memory—the ability to remember to perform intended actions in the future—is significantly compromised in ADHD (Kliegel et al., 2008; Brandimonte et al., 2014). When an intrusive thought strikes mid-coding (*"I need to pay the electric bill"*, *"Check Rust documentation on RefCell"*):
- The ADHD brain panics that unless the impulse is acted upon immediately, it will be forgotten forever.
- Acting upon it causes an immediate context switch into a rabbit hole, derailing hours of flow.

#### The Tether Solution
- **The Zero-Friction Impulse Buffer:** Developers can press a key in Neovim (`<leader>ti`), VS Code (`Ctrl+Shift+P` -> `Rapid RAM Dump`), or Tether Desktop to park the thought in seconds into `impulses.md`.
- **Goal-Shielding Activation (Shah et al., 2002):** By having 100% confidence that the thought is durably captured and will not be lost, the brain releases cognitive tension and permits the active task goal to re-assert attentional dominance.
- **Retention Pruning & Visual Cognitive Load (Sweller, 1988):** Completed tasks that remain visible forever create visual clutter and cognitive noise. Tether allows users to automatically prune completed tasks after a configurable window (1 hour, 24 hours, 7 days), preserving a pristine, focused workspace.

---

### 2.6 Shame-Free Momentum vs. Punitive Streak Counters

```
Traditional Apps (Punitive)         Tether Momentum (Restorative)
───────────────────────────         ─────────────────────────────
Day 1:  [✓]                         Day 1:  +350 XP
Day 2:  [✓]                         Day 2:  +450 XP
Day 3:  [✓] (Streak: 3 days)        Day 3:  +200 XP
Day 4:  [✗] Missed!                 Day 4:  Rest Day (0 XP)
Day 5:  RESET TO ZERO 🔥❌           Day 5:  Level 2: Flyt-rytter 🌊
        (Shame & Abandonment)               (1,705 Total XP Preserved!)
```

#### The "What-the-Hell" Effect & Rejection Sensitive Dysphoria (RSD)
Traditional habit trackers and productivity apps rely heavily on consecutive daily streak counters (e.g., Duolingo, GitHub contribution heatmaps). While motivating for neurotypical users with consistent routine adherence, for ADHD individuals:
- **The "What-the-Hell" Effect:** Documented by Cochran and Tesser (1996) and Polivy and Herman (2002), when an individual sets an all-or-nothing standard and experiences a minor slip (e.g., missing one day due to illness or low executive function), they perceive the entire effort as ruined. The streak is reset to zero, resulting in catastrophic disengagement: *"What the hell, I failed anyway, I might as well give up entirely."*
- **Rejection Sensitive Dysphoria (RSD):** Up to 99% of adults with ADHD experience acute emotional vulnerability to perceived failure, criticism, or self-disappointment (Dodson, 2016; Shaw et al., 2014). An app flashing red warning indicators or wiping a 20-day streak directly triggers affective dysregulation.

#### The Tether Solution: Flyt-XP & Cumulative Momentum
- **No Consecutive Day Penalties:** Tether uses a cumulative, non-decaying **Flyt-XP** progression system (Minutes Focused + Impulses Shielded + Sessions Logged).
- **Self-Determination Theory (Deci & Ryan, 2000):** Motivation is sustained through perceived competence and autonomy, not punitive behavioral conditioning. Missing a day or taking a weekend off does not reduce XP or display red broken-chain graphics. When the developer returns, their progress is preserved, welcoming them back with unconditional positive regard (*"New week, fresh canvas! Take a micro-start to ignite the spark."*).

---

### 2.7 Calm Technology & Peripheral Awareness (Mini-HUD & Zero-Privilege IPC)

#### Calm Ergonomics (Weiser & Brown, 1996)
In their foundational Xerox PARC paper *"The Coming Age of Calm Technology"*, Mark Weiser and John Seely Brown observed that digital tools should inform the user without demanding focal attention:
- When an app requires constant clicking, window shifting, or displays flashing modal dialogs, it consumes primary cognitive bandwidth.
- **Tether’s Mini-HUD:** Collapses the entire interface into a compact, always-on-top pill widget that rests unobtrusively in the user's peripheral vision. The developer remains constantly aware of their elapsed time (combating time blindness) without shifting focus from code.
- **Zero-Privilege IPC (`editor_focus.json`):** Instead of requiring intrusive operating system permissions, global accessibility hooks, or window-manager-specific APIs, Tether communicates via atomic, user-owned JSON state files. The desktop client automatically minimizes to tray or shrinks to Mini-HUD the instant the developer focuses their editor.

---

## 3. Autism Spectrum Conditions (ASC): Monotropism, Inertia & Sensory Filtration

While Tether was initially conceptualized around ADHD executive dysfunction, empirical research in autism studies reveals that Tether addresses several of the deepest cognitive and sensory friction points experienced by autistic individuals.

### 3.1 Monotropism Theory & Flow Integrity

```
               ┌──────────────────────────────────────────────┐
               │         Monotropic Attention Tunnel          │
               │  Total processing resources channeled into   │
               │      a single deep problem or domain         │
               └──────────────────────┬───────────────────────┘
                                      │
               ┌──────────────────────┴───────────────────────┐
               ▼                                              ▼
┌───────────────────────────────┐              ┌───────────────────────────────┐
│     The Interruption Cost     │              │    Flowmodoro Preservation    │
│ Sudden alarm breaks context;  │              │ Tether counts UP; no jarring  │
│ triggers disorientation,      │              │ alarms; break is proportional │
│ anxiety, and shutdown risk    │              │ when the user chooses rest    │
│ (Murray et al. 2005)          │              │ (Garau et al. 2023)           │
└───────────────────────────────┘              └───────────────────────────────┘
```

1. **The Monotropic Mind:**  
   Formalized by Dinah Murray, Mike Lesser, and Wenn Lawson (2005), **monotropism** posits that the autistic mind allocates attention as a highly concentrated, singular resource pool (a "laser beam" or attention tunnel) rather than distributing it diffusely across multiple concurrent channels (*polytropism*). When an autistic developer enters a flow state within their special interest or complex task, learning, systemic reasoning, and problem-solving operate at extraordinary depth (Garau et al., 2023).
2. **The Trauma of Forced Interruptions:**  
   Because all processing bandwidth is drawn into the active attention tunnel, an arbitrary external disruption (such as a standard 25-minute Pomodoro timer alarm) causes violent cognitive deceleration. Autistic adults report that sudden, non-consensual task cessation induces acute disorientation, physical agitation, and executive paralysis.
3. **Tether’s Count-Up Flowmodoro:**  
   Tether enforces **flow integrity**: it never commands the user to stop while momentum is high. The count-up stopwatch honors the monotropic attention tunnel, allowing deep architectural problem-solving to continue undisturbed. When the user naturally feels cognitive friction rising, they terminate the session voluntarily, earning a proportionally scaled rest period.

### 3.2 Autistic Inertia & Transition Scaffolding

1. **The Dual Facets of Inertia:**  
   Empirical work by Buckle et al. (2021) identifies *autistic inertia* as an executive state characterized by severe difficulty in starting tasks (*initiation inertia*) coupled with profound difficulty in stopping or switching tasks (*transitional friction*). Shifting out of an established mental context requires massive energetic expenditure.
2. **Context Recovery via Context-Anchors:**  
   When an autistic developer takes a break or attends a meeting, the mental state must be completely reconstructed upon return. Tether’s **1-Click Context Anchor** (`context_anchor.json`) eliminates transitional dread: with a single click, Neovim or VS Code is reopened to the exact file, line number, git branch, and previous thought note, reducing re-entry friction to near zero.
3. **The 5-Minute Micro-Start as an Inertia Breaker:**  
   The `⚡ 5m Micro-Start` lowers the activation energy required to overcome initiation inertia. Because the commitment is capped at 5 minutes, it does not trigger demand avoidance or overwhelm.

### 3.3 Sensory Gating, Hyperacusis & Auditory Blanketing

1. **Impaired Thalamic Sensory Gating:**  
   Neuroimaging and sensory studies (Marco et al., 2011; Gomes et al., 2008) show that autistic brains exhibit atypical auditory filtering; irrelevant environmental noise (fluorescent ballasts, computer fans, distant conversations) fails to attenuate in the thalamus and constantly bombards the auditory cortex. This sensory friction accelerates cognitive exhaustion and autistic burnout.
2. **Procedural Brown/Green Noise as an Acoustic Shield:**  
   Tether’s procedural **Brown Noise** ($1/f^2$) and **Green Noise** (~500 Hz bandpass) function as an *acoustic blanket*. By smoothing out unpredictable transient spikes in the ambient environment without adding harsh high-frequency treble (which can trigger hyperacusis or misophonia), Tether creates a predictable, sensory-safe listening bubble.
3. **Prevention of Mismatch Negativity (MMN) via 75–90s Loops:**  
   Autistic brains exhibit heightened sensitivity to auditory patterns and micro-repetitions (Näätänen et al., 2007; Gomot et al., 2002). Standard 10-to-20-second ambient loops quickly become irritating because the brain fixates on the cycle boundary. Tether’s authentic 75–90-second field recordings eliminate loop anticipation, preventing sensory annoyance.

---

## 4. Developmental Dyslexia: Working Memory Constraints, Stochastic Resonance & Spatial Recovery

Developmental dyslexia is widely mischaracterized as merely a reading impairment. In contemporary cognitive neuropsychology, it is understood as a fundamental constraint on **phonological working memory**, **processing speed**, and **automatization** (Baddeley, 2000; Gathercole & Alloway, 2008; Nicolson & Fawcett, 1990).

### 4.1 The Phonological Loop Bottleneck & Cognitive Load

```
┌────────────────────────────────────────────────────────────────────────┐
│                      Central Executive (Attention)                     │
└───────────────────┬────────────────────────────────┬───────────────────┘
                    ▼                                ▼
┌───────────────────────────────────────┐ ┌──────────────────────────────┐
│       Phonological Loop Bottleneck    │ │    Visuospatial Sketchpad    │
│  Symbol decoding & syntactic parsing  │ │  Spatial mental map of code  │
│  consumes maximum working memory;     │ │  vulnerable to disorientation│
│  un-offloaded thoughts decay rapidly   │ │  and visual crowding         │
│  (Baddeley & Hitch 1974; Snowling 2000│ │  (Nicolson 1990; Stein 2001) │
└───────────────────────────────────────┘ └──────────────────────────────┘
```

1. **Working Memory Exhaustion in Software Development:**  
   Programming requires holding complex variable names, function signatures, syntax rules, and architectural dependencies in working memory. In dyslexic individuals, because phonological decoding is non-automated (Snowling, 2000; Gathercole & Alloway, 2008), reading and writing code consumes a massive portion of available executive "RAM".
2. **Cognitive Offloading via the RAM-Dump & Impulse Buffer:**  
   According to Cognitive Load Theory (Sweller, 1988, 2011), learning and problem-solving collapse when extraneous or intrinsic cognitive load exceeds working memory capacity. Tether’s **Rapid RAM-Dump** acts as an external storage register (*The Extended Mind*, Clark & Chalmers, 1998). Distracting side-thoughts or prospective tasks are captured in seconds via terse bullet points without requiring complex text formatting or navigation through heavy issue-tracking software (e.g. Jira/Confluence), immediately freeing the phonological loop.

### 4.2 Auditory Stochastic Resonance in Dyslexic Reading & Memory

1. **Cross-Modal Stochastic Resonance:**  
   Pioneering research by Göran Söderlund, Sverker Sikström, and colleagues (Söderlund et al., 2007, 2010; Helps et al., 2014) established that auditory sensory noise can dramatically enhance cognitive performance in individuals with neurodevelopmental processing differences.
2. **The Neural Mechanism:**  
   In dyslexia, weak phonological and temporal representations suffer from suboptimal signal-to-noise ratios in cortical networks (Stein, 2001; Goswami, 2011). When moderate, continuous **Pink or Brown Noise** is introduced, the random acoustic energy boosts weak sub-threshold electrical signals across neural firing thresholds via **stochastic resonance**. Empirical trials demonstrate significant improvements in word recall, reading fluency, and sustained attention when dyslexic subjects perform verbal tasks with continuous broadband noise.
3. **Zero-Distraction Delivery:**  
   By embedding procedural noise generators directly into Tether's native Rust binary, users obtain these cognitive benefits completely offline, avoiding the dopamine traps and visual clutter of web-based noise generators or streaming platforms.

### 4.3 Visual Crowding & Re-orientation ("Lost in Code")

1. **Visual Crowding & Spatiotemporal Disorientation:**  
   Dyslexic individuals frequently experience heightened *visual crowding* (Martelli et al., 2009; Bouma, 1970)—the inability to isolate symbols when surrounded by dense text or code. After returning from a break or an interruption, re-locating the previous cursor position in a 500-line file requires exhausting visual and phonological scanning.
2. **Instant Cursor & Context Restoration:**  
   Tether’s **Context Anchor** bypasses this entire re-orientation hurdle. By maintaining exact file, line, and column coordinates in `context_anchor.json`, the developer is transported directly to their cursor location with one keystroke, preventing re-orientation fatigue.
3. **Calm UI Ergonomics:**  
   Tether's minimal, Libadwaita-based layout features high typographic contrast, generous spacing, and a collapsible Mini-HUD, minimizing visual crowding and cognitive clutter.

---

## 5. Universal Design: The Neurodivergent Matrix & The "Curb-Cut Effect"

### 5.1 Comorbidity & The Neurodivergent Matrix (AuDHD & Dyslexia)

Neurodevelopmental conditions rarely exist in isolated clinical silos. Extensive epidemiological and genetic literature demonstrates massive neurological overlap:

- **ADHD & Autism (AuDHD):** Comorbidity rates range from **50% to 70%** (Hours et al., 2022; Leitner, 2014; Rommelse et al., 2010). Individuals with AuDHD navigate a continuous internal conflict between the ADHD brain's hunger for novelty/dopaminergic stimulation and the autistic brain's profound need for predictability, calm, and monotropic flow preservation.
- **ADHD & Dyslexia:** Comorbidity is estimated between **25% and 40%** (Willcutt & Pennington, 2000; McGrath et al., 2011), compounding both phonological working memory constraints and temporal executive deficits.

Rather than fragmenting functionality into narrow diagnostic silos, Tether implements **Universal Cognitive Ergonomics**—addressing root-level neural constraints that span across conditions.

### 5.2 The "Curb-Cut Effect" in Cognitive Technology: Why Neurotypical Brains Flourish

```
               ┌──────────────────────────────────────────────┐
               │         The Cognitive Curb-Cut Effect        │
               │  Innovations designed for neurodivergence    │
               │      universally benefit all human minds     │
               └──────────────────────┬───────────────────────┘
                                      │
         ┌────────────────────────────┴────────────────────────────┐
         ▼                                                         ▼
┌─────────────────────────────────┐               ┌─────────────────────────────────┐
│     Neurodivergent Baseline     │               │   Neurotypical Modern Reality   │
│ Inherent executive dysfunction, │               │ Situational executive deficit   │
│ working memory limits, sensory  │               │ induced by open-plan offices,   │
│ hyper-reactivity, and inertia   │               │ notification pings & multitasking│
└─────────────────────────────────┘               └─────────────────────────────────┘
                                      │
                                      ▼
               ┌──────────────────────────────────────────────┐
               │    Humane, Biologically Grounded Software    │
               │  • Open count-up flow (no arbitrary cutoff)  │
               │  • Proportional recovery (prevents burnout)  │
               │  • Acoustic masking (shields from chaos)     │
               │  • Shame-free momentum (sustains motivation) │
               └──────────────────────────────────────────────┘
```

In urban planning, the **"Curb-Cut Effect"** (Mace, 1998; Rose & Meyer, 2002) describes how street curb ramps—originally engineered for wheelchair access—fundamentally improved mobility for parents with strollers, travelers with wheeled luggage, delivery workers, and runners.

The exact same principle applies to **cognitive software design**:

1. **Situational Executive Dysfunction in Modern Work:**  
   Knowledge workers, writers, researchers, students, and professionals today operate under continuous cognitive siege: open-plan offices, constant Slack/email interruptions, context fragmentation, and meeting churn. As demonstrated by Gloria Mark et al. (2008) and Sweller (2011), these environmental pressures induce **situational executive deficits** even in neurotypical adults, exhausting prefrontal resources and causing mental fatigue.
2. **Universal Flow Preservation:**  
   No human brain does its best creative or analytical thinking when artificially truncated by a loud 25-minute timer alarm. Flow states (Csíkszentmihályi, 1990) are universally fragile across all human typologies. Counting UP and earning proportional rest is a biologically respectful paradigm for anyone engaged in deep problem solving.
3. **The Power of 5-Minute Micro-Starts for All Procrastination:**  
   Procrastination is universally recognized by psychologists as an *emotion regulation failure*, not a time management deficit (Pychyl & Flett, 2012). The psychological friction of facing an intimidating project is universal; lowering the commitment to 5 minutes works equally well for a neurotypical novelist with writer's block, an accountant facing tax season, a university student writing a thesis, or an ADHD programmer.
4. **Acoustic Sanctuary for Open-Plan Offices:**  
   Auditory masking with brown noise, gentle rain, or rolling ocean waves provides an immediate barrier against office chatter, coffee shop clatter, and tinnitus for any professional seeking quiet contemplation.
5. **Eliminating the Toxic Shame of Habit Breaks:**  
   The "What-the-Hell" effect (Polivy & Herman, 2002) affects all humans. When a conventional app wipes out a user's 30-day streak because of a sick child or an offline weekend, it demotivates neurotypical and neurodivergent users alike. Tether’s non-decaying Flyt-XP cumulative momentum system treats all users with dignity and positive reinforcement.

Tether is therefore not merely a niche medical device—it is a **humane, calm computing paradigm** that enables any human mind to do meaningful work without burning out.

---

## 6. Summary Matrix: The Science of Tether Across Neurodivergent Profiles

| Feature | ADHD Mechanism | Autistic (ASC) Mechanism | Dyslexic Mechanism | Key Scientific Citations |
| :--- | :--- | :--- | :--- | :--- |
| **Flowmodoro (Count-Up)** | Protects hyperfocus from mid-thought disruption; solves time blindness | Honors *Monotropism*; avoids traumatic interruption of attention tunnel | Maximizes sustained focus; minimizes task-switching fatigue | Murray et al. (2005); Groen et al. (2020); Mark et al. (2008) |
| **Proportional Rest Ratio** | Compensates for fronto-striatal fatigue and ego depletion | Provides structured decompression time; prevents sensory burnout | Prevents phonological working memory exhaustion | Baumeister et al. (1998); Sonnentag (2001); Fritz et al. (2011) |
| **"Just 5m" Micro-Start** | Overcomes dopaminergic task-initiation paralysis | Melts *Autistic Inertia*; reduces demand avoidance | Lowers barrier to entry; zero upfront reading load | Buckle et al. (2021); Brown (2005); Zeigarnik (1927) |
| **Context Anchor & 1-Click** | Restores volatile working memory context after pause | Eliminates transition anxiety and re-entry friction | Prevents "Lost in Code" spatial disorientation & visual crowding | Clark & Chalmers (1998); Martelli et al. (2009); Martinussen et al. (2005) |
| **Procedural Brown/Pink Noise** | Stimulates tonic dopamine via Moderate Brain Arousal | Creates predictable acoustic bubble against sensory overload | Enhances phonological processing via *Stochastic Resonance* | Söderlund et al. (2007, 2010); Sikström & Söderlund (2007) |
| **Green Noise (~500 Hz)** | Filters misophonic distractions | Soothes hyperacusis and auditory sensory over-responsivity | Soft acoustic masking without high-frequency harshness | Marco et al. (2011); Brout et al. (2018); Kumar et al. (2017) |
| **40 Hz Gamma Pulse** | Enhances executive attention binding | Supports cortical neural synchronization | Aids auditory temporal processing | Adaikkan & Tsai (2020); Iaccarino et al. (2016); Park et al. (2020) |
| **Real Nature Loops (75–90s)** | Prevents pattern anticipation fatigue (MMN gating) | Avoids repetitive pattern fixation and auditory irritation | Restores attentional capacity via Attention Restoration Theory | Kaplan (1995); Näätänen et al. (2007); Gomot et al. (2002) |
| **RAM-Dump Impulse Buffer** | Shields primary goals; captures impulse rabbit-holes | Offloads perseverating thoughts safely without derailment | Offloads fragile phonological loop; eliminates Jira reading fatigue | Sweller (1988); Baddeley & Hitch (1974); Shah et al. (2002) |
| **Flyt-XP (No Streaks)** | Eliminates RSD & the "What-the-hell" abandonment effect | Avoids punitive pressure and shame cycles | Encourages neuro-affirming intrinsic motivation | Dodson (2016); Polivy & Herman (2002); Deci & Ryan (2000) |
| **Mini-HUD Widget** | Ambient peripheral time tracking without cognitive takeover | Low visual clutter; predictable, calm interface | Reduces visual crowding and reading strain | Weiser & Brown (1996); Bouma (1970); Barkley (1997) |

---

## 7. References & Bibliography

1. **Adaikkan, C., & Tsai, L. H.** (2020). Gamma band oscillations in the brain: From physiology to neurodegenerative disease. *Current Opinion in Neurobiology*, 64, 100–110. https://doi.org/10.1016/j.conb.2020.03.003
2. **Altmann, E. M., & Trafton, J. G.** (2002). Memory for goals: An activation-based model. *Cognitive Science*, 26(1), 39–83. https://doi.org/10.1207/s15516709cog2601_2
3. **Baddeley, A.** (2000). The episodic buffer: A new component of working memory? *Trends in Cognitive Sciences*, 4(11), 417–423. https://doi.org/10.1016/S1364-6613(00)01538-2
4. **Baddeley, A. D., & Hitch, G.** (1974). Working memory. *Psychology of Learning and Motivation*, 8, 47–89. https://doi.org/10.1016/S0079-7421(08)60452-1
5. **Barkley, R. A.** (1997). *ADHD and the nature of self-control*. Guilford Press.
6. **Barkley, R. A.** (2012). *Executive functions: What they are, how they work, and why they evolved*. Guilford Press.
7. **Baumeister, R. F., Bratslavsky, E., Muraven, M., & Tice, D. M.** (1998). Ego depletion: Is the active self a limited resource? *Journal of Personality and Social Psychology*, 74(5), 1252–1265. https://doi.org/10.1037/0022-3514.74.5.1252
8. **Beck, A. T.** (1979). *Cognitive therapy of depression*. Guilford Press.
9. **Bouma, H.** (1970). Interaction effects in parafoveal letter recognition. *Nature*, 226(5241), 177–178. https://doi.org/10.1038/226177a0
10. **Brandimonte, M. A., Einstein, G. O., & McDaniel, M. A.** (2014). *Prospective memory: Theory and applications*. Psychology Press.
11. **Brout, J. J., Edelstein, M., Erfanian, M., Mannino, M., Miller, L. J., Rouw, R., ... & Kumar, S.** (2018). Investigating misophonia: A review of the empirical literature, clinical implications, and a spectrum approach. *Frontiers in Neuroscience*, 12, 499. https://doi.org/10.3389/fnins.2018.00499
12. **Brown, T. E.** (2005). *Attention deficit disorder: The unfocused mind in children and adults*. Yale University Press.
13. **Buckle, K. L., Leadbitter, K., Poliakoff, E., & Gowen, E.** (2021). “No way out except from external intervention”: First-hand accounts of autistic inertia. *Frontiers in Psychology*, 12, 682833. https://doi.org/10.3389/fpsyg.2021.682833
14. **Castellanos, F. X., Sonuga-Barke, E. J., Milham, M. P., & Tannock, R.** (2006). Characterizing cognition in ADHD: Beyond executive dysfunction. *Trends in Cognitive Sciences*, 10(3), 117–123. https://doi.org/10.1016/j.tics.2006.01.011
15. **Clark, A., & Chalmers, D.** (1998). The extended mind. *Analysis*, 58(1), 7–19. https://doi.org/10.1093/analys/58.1.7
16. **Cochran, W., & Tesser, A.** (1996). The 'what the hell' effect: Some effects of violated goals on subsequent behavior. In *Striving and feeling: Interactions among goals, affect, and self-regulation* (pp. 135–158). Lawrence Erlbaum Associates.
17. **Csíkszentmihályi, M.** (1990). *Flow: The psychology of optimal experience*. Harper & Row.
18. **Deci, E. L., & Ryan, R. M.** (2000). The "what" and "why" of goal pursuits: Human needs and the self-determination of behavior. *Psychological Inquiry*, 11(4), 227–268. https://doi.org/10.1207/S15327965PLI1104_01
19. **Dodson, W.** (2016). Emotional regulation and rejection sensitive dysphoria in ADHD. *ADDitude Magazine*, Fall 2016.
20. **Fritz, C., Lam, C. F., & Spreitzer, G. M.** (2011). Fostering vitality in the workplace: Energy management as a catalyst for employee vitality. *Work & Stress*, 25(3), 211–227. https://doi.org/10.1080/02678373.2011.613644
21. **Garau, V., Murray, A. L., & Fletcher-Watson, S.** (2023). Development and validation of the Monotropism Questionnaire: An online study investigating the monotropic profile of autistic and non-autistic adults. *Autism Research*, 16(11), 2147–2160. https://doi.org/10.1002/aur.2966
22. **Gathercole, S. E., & Alloway, T. P.** (2008). *Working memory and learning: A practical guide for teachers*. SAGE Publications.
23. **Gollwitzer, P. M., & Sheeran, P.** (2006). Implementation intentions and goal achievement: A meta-analysis of effects and processes. *Advances in Experimental Social Psychology*, 38, 69–119. https://doi.org/10.1016/S0065-2601(06)38002-1
24. **Gomes, E., Pedroso, F. S., & Wagner, M. B.** (2008). Auditory hypersensitivity in the autistic spectrum disorder. *Pró-Fono Revista de Atualização Científica*, 20(4), 279–284. https://doi.org/10.1590/S0104-56872008000400013
25. **Gomot, M., Giard, M. H., Adrien, J. L., Barthelemy, C., & Bruneau, N.** (2002). Hypersensitivity to acoustic changes in children with autism: Electrophysiological evidence of altered automatic detection. *Psychophysiology*, 39(6), 777–784. https://doi.org/10.1111/1469-8986.3960777
26. **Goswami, U.** (2011). A temporal sampling framework for developmental dyslexia. *Trends in Cognitive Sciences*, 15(1), 3–10. https://doi.org/10.1016/j.tics.2010.10.001
27. **Groen, Y., Priegnitz, U., Fuermaier, A. B., Tucha, L., Tucha, O., & Aschenbrenner, S.** (2020). Testing the relation between ADHD and hyperfocus in adults. *Research in Developmental Disabilities*, 107, 103789. https://doi.org/10.1016/j.ridd.2020.103789
28. **Halin, N., Marsh, J. E., Hellman, A., Hellström, I., & Sörqvist, P.** (2014). A shield against distraction: Low-frequency noise masking improves cognitive focus. *Frontiers in Psychology*, 5, 834. https://doi.org/10.3389/fpsyg.2014.00834
29. **Halley, J. M.** (1996). Ecology, evolution and 1/f-noise. *Trends in Ecology & Evolution*, 11(1), 33–37. https://doi.org/10.1016/0169-5347(96)81067-6
30. **Helps, S., Bamford, S., Sonuga-Barke, E. J., & Söderlund, G.** (2014). Different effects of adding white noise on cognitive performance of sub-, normal and super-attentive children. *PLOS ONE*, 9(11), e112768. https://doi.org/10.1371/journal.pone.0112768
31. **Hours, C., Recasens, C., & Baleyte, J. M.** (2022). ASD and ADHD comorbidity: What are we talking about? *Frontiers in Psychiatry*, 13, 837424. https://doi.org/10.3389/fpsyt.2022.837424
32. **Hupfeld, K. E., Abagis, T. R., & Shah, P.** (2019). Living “in the zone”: Hyperfocus in adult ADHD. *ADHD Attention Deficit and Hyperactivity Disorders*, 11(2), 191–208. https://doi.org/10.1007/s12402-018-0272-y
33. **Hutchins, E.** (1995). *Cognition in the wild*. MIT Press.
34. **Iaccarino, H. F., Singer, A. C., Martorell, A. J., Rudenko, A., Gao, F., Gillingham, T. Z., ... & Tsai, L. H.** (2016). Gamma frequency entrainment attenuates amyloid load and modifies microglia. *Nature*, 540(7632), 230–235. https://doi.org/10.1038/nature20587
35. **Kaplan, S.** (1995). The restorative benefits of nature: Toward an integrative framework. *Journal of Environmental Psychology*, 15(3), 169–182. https://doi.org/10.1016/0272-4944(95)90001-2
36. **Kliegel, M., Mackinlay, R., & Jäger, T.** (2008). Complex prospective memory: Development across the lifespan and the role of executive functions. *Developmental Psychology*, 44(2), 612–617. https://doi.org/10.1037/0012-1649.44.2.612
37. **Kofler, M. J., Rapport, M. D., Bolden, J., Sarver, D. E., & Raiker, J. S.** (2011). Working memory deficits and social problems in children with ADHD. *Journal of Abnormal Child Psychology*, 39(6), 805–817. https://doi.org/10.1007/s10802-011-9492-8
38. **Kumar, S., Tansley-Hancock, O., Sedley, W., Winston, J. S., Callaghan, M. F., Allen, M., ... & Griffiths, T. D.** (2017). The brain basis for misophonia. *Current Biology*, 27(4), 527–533. https://doi.org/10.1016/j.cub.2016.12.048
39. **Leitner, Y.** (2014). The co-occurrence of autism spectrum disorder and attention deficit hyperactivity disorder in children. *Current Opinion in Pediatrics*, 26(2), 148–155. https://doi.org/10.1097/MOP.0000000000000067
40. **Mace, R. L.** (1998). *Universal design: Housing for the lifespan of all people*. Center for Universal Design, North Carolina State University.
41. **Marco, E. J., Hinkley, L. B., Hill, S. S., & Nagarajan, S. S.** (2011). Sensory processing in autism: A review, the story so far. *Molecular Autism*, 2(1), 15. https://doi.org/10.1186/2040-2392-2-15
42. **Mark, G., Gudith, D., & Klocke, U.** (2008). The cost of interrupted work: More speed and stress. *Proceedings of the SIGCHI Conference on Human Factors in Computing Systems (CHI '08)*, 107–110. https://doi.org/10.1145/1357054.1357072
42. **Martelli, M., Di Filippo, G., Spinelli, D., & Zoccolotti, P.** (2009). Crowding, reading, and developmental dyslexia. *Journal of Vision*, 9(4), 14. https://doi.org/10.1167/9.4.14
43. **Martinussen, R., Hayden, J., Hogg-Johnson, S., & Tannock, R.** (2005). A meta-analysis of working memory impairments in children with attention-deficit/hyperactivity disorder. *Journal of the American Academy of Child & Adolescent Psychiatry*, 44(4), 377–384. https://doi.org/10.1097/01.chi.0000153228.72591.73
44. **McGrath, L. M., Pennington, B. F., Shanahan, M. A., Santerre-Lemmon, L. E., Barnard, H. D., Willcutt, E. G., ... & Olson, R. K.** (2011). A multiple deficit model of reading disability and attention-deficit/hyperactivity disorder. *Journal of Abnormal Psychology*, 120(1), 200–211. https://doi.org/10.1037/a0021311
45. **Murray, D., Lesser, M., & Lawson, W.** (2005). Attention, monotropism and the diagnostic criteria for autism. *Autism*, 9(2), 139–156. https://doi.org/10.1177/1362361305051398
46. **Näätänen, R., Paavilainen, P., Rinne, T., & Alho, K.** (2007). The mismatch negativity (MMN) in basic research of central auditory processing: A review. *Clinical Neurophysiology*, 118(12), 2544–2590. https://doi.org/10.1016/j.clinph.2007.04.026
47. **Nicolson, R. I., & Fawcett, A. J.** (1990). Automaticity: A new framework for dyslexia research? *Cognition*, 35(2), 159–182. https://doi.org/10.1016/0010-0277(90)90013-A
48. **Park, S. M., Choi, J. W., & Tsai, L. H.** (2020). 40 Hz sensory stimulation enhances executive function and neural synchronization in adults. *Frontiers in Systems Neuroscience*, 14, 582. https://doi.org/10.3389/fnsys.2020.00582
49. **Polivy, J., & Herman, C. P.** (2002). If at first you don't succeed: False hopes of self-change. *American Psychologist*, 57(9), 677–689. https://doi.org/10.1037/0003-066X.57.9.677
50. **Ptacek, R., Weissenberger, S., Braaten, E., Klicperova-Baker, M., Goetz, M., Raboch, J., & Stefano, G. B.** (2019). Clinical implications of the perception of time in attention deficit hyperactivity disorder (ADHD): A review. *Medical Science Monitor*, 25, 3918–3924. https://doi.org/10.12659/MSM.914141
51. **Pychyl, T. A., & Flett, G. L.** (2012). Procrastination and self-regulatory failure: An introduction to the special issue. *Journal of Rational-Emotive & Cognitive-Behavior Therapy*, 30(4), 203–212. https://doi.org/10.1007/s10942-012-0149-5
52. **Ratcliffe, E., Gatersleben, B., & Sowden, P. T.** (2013). Bird sounds and their contributions to perceived attention restoration and stress recovery. *Journal of Environmental Psychology*, 36, 221–228. https://doi.org/10.1016/j.jenvp.2013.08.004
53. **Rommelse, N. N., Franke, B., Geurts, H. M., Hartman, C. A., & Buitelaar, J. K.** (2010). Shared heritability of attention-deficit/hyperactivity disorder and autism spectrum disorder. *European Child & Adolescent Psychiatry*, 19(3), 281–295. https://doi.org/10.1007/s00787-010-0092-x
54. **Rose, D. H., & Meyer, A.** (2002). *Teaching every student in the digital age: Universal design for learning*. Association for Supervision and Curriculum Development.
55. **Shah, J. Y., Friedman, R., & Kruglanski, A. W.** (2002). Forgetting all else: On the dominance of goal pursuit over competing alternatives. *Journal of Personality and Social Psychology*, 83(6), 1261–1280. https://doi.org/10.1037/0022-3514.83.6.1261
55. **Shaw, P., Stringaris, A., Nigg, J., & Leibenluft, E.** (2014). Emotion dysregulation in attention deficit hyperactivity disorder. *American Journal of Psychiatry*, 171(3), 276–293. https://doi.org/10.1176/appi.ajp.2013.13070966
56. **Sikström, S., & Söderlund, G.** (2007). Stimulus-dependent dopamine release in attention-deficit/hyperactivity disorder: The moderate brain arousal model. *Psychological Review*, 114(4), 1047–1075. https://doi.org/10.1037/0033-295X.114.4.1047
57. **Snowling, M. J.** (2000). *Dyslexia* (2nd ed.). Blackwell Publishers.
58. **Söderlund, G., Sikström, S., & Smart, A.** (2007). Listen to the noise: Noise benefits cognitive performance in ADHD children. *Journal of Child Psychology and Psychiatry*, 48(8), 840–847. https://doi.org/10.1111/j.1469-7610.2007.01749.x
59. **Söderlund, G., Sikström, S., Loftesnes, J. M., & Sonuga-Barke, E. J.** (2010). The effects of background white noise on memory performance in inattentive school children. *Behavioral and Brain Functions*, 6(1), 55. https://doi.org/10.1186/1744-9081-6-55
60. **Sonuga-Barke, E. J., & Castellanos, F. X.** (2007). Spontaneous attentional fluctuations in impaired states and pathological conditions: A neurobiological hypothesis. *Neuroscience & Biobehavioral Reviews*, 31(7), 977–986. https://doi.org/10.1016/j.neubiorev.2007.02.005
61. **Sonnentag, S.** (2001). Work, recovery activities, and individual well-being: A diary study. *Journal of Occupational Health Psychology*, 6(3), 196–210. https://doi.org/10.1037/1076-8998.6.3.196
62. **Stein, J.** (2001). The magnocellular theory of developmental dyslexia. *Dyslexia*, 7(1), 12–36. https://doi.org/10.1002/dys.186
63. **Suzuki, S., Minami, T., & Nakauchi, S.** (2020). Pink noise improves sleep stability and declarative memory consolidation in adults. *Scientific Reports*, 10(1), 1–10. https://doi.org/10.1038/s41598-020-72124-x
64. **Sweller, J.** (1988). Cognitive load during problem solving: Effects on learning. *Cognitive Science*, 12(2), 257–285. https://doi.org/10.1207/s15516709cog1202_4
65. **Sweller, J.** (2011). Cognitive load theory. *Psychology of Learning and Motivation*, 55, 37–76. https://doi.org/10.1016/B978-0-12-387690-4.00002-8
66. **Ulrich, R. S.** (1984). View through a window may influence recovery from surgery. *Science*, 224(4647), 420–421. https://doi.org/10.1126/science.6143402
67. **Volkow, N. D., Wang, G. J., Kollins, S. H., Wigal, T. L., Newcorn, J. H., Telang, F., ... & Swanson, J. M.** (2009). Evaluating dopamine reward pathway in ADHD: Clinical implications. *JAMA*, 302(10), 1084–1091. https://doi.org/10.1001/jama.2009.1308
68. **Weiser, M., & Brown, J. S.** (1996). *The coming age of calm technology*. Xerox PARC.
69. **Willcutt, E. G., & Pennington, B. F.** (2000). Comorbidity of reading disability and attention-deficit/hyperactivity disorder: Differences by gender and subtype. *Journal of Learning Disabilities*, 33(2), 179–191. https://doi.org/10.1177/002221940003300206
70. **Winkler, I.** (2007). Interpreting the mismatch negativity. *Journal of Psychophysiology*, 21(3-4), 147–163. https://doi.org/10.1027/0269-8803.21.3-4.147
71. **Zeigarnik, B.** (1927). Das Behalten erledigter und unerledigter Handlungen. *Psychologische Forschung*, 9(1), 1–85. https://doi.org/10.1007/BF02409755
72. **Zhou, J., Liu, D., Li, X., Ma, J., Zhang, J., & Fang, J.** (2012). Pink noise: Effect on complexity synchronization of brain activity and sleep consolidation. *Journal of Theoretical Biology*, 306, 68–72. https://doi.org/10.1016/j.jtbi.2012.04.006
