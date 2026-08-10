/// Replace LaTeX math commands with Unicode symbols.
///
/// Uses a single-pass scanner for O(n) performance instead of
/// O(n·m) sequential `String::replace` calls.
pub fn replace_math_symbols(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut iter = text.chars().peekable();

    while let Some(ch) = iter.next() {
        if ch == '\\' {
            match iter.peek() {
                Some(&'\\') => {
                    iter.next();
                    result.push(' ');
                }
                Some(&'{') => {
                    iter.next();
                    result.push('{');
                }
                Some(&'}') => {
                    iter.next();
                    result.push('}');
                }
                Some(&',') | Some(&';') | Some(&':') | Some(&'>') | Some(&'~') => {
                    iter.next();
                    result.push(' ');
                }
                Some(&'!') => {
                    iter.next(); // negative space — remove
                }
                Some(&next_ch) if next_ch.is_alphabetic() => {
                    let mut name = String::new();
                    name.push(next_ch);
                    iter.next();

                    while let Some(&c) = iter.peek() {
                        if c.is_alphabetic() || c == '*' {
                            name.push(c);
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    // Text commands: strip the command and consume the braced argument
                    if matches!(
                        name.as_str(),
                        "text" | "mathrm" | "mathscr" | "mathnormal" | "boldsymbol" | "pmb"
                        | "operatorname"
                    ) {
                        if let Some(&'{') = iter.peek() {
                            iter.next(); // consume opening brace
                            while let Some(&c) = iter.peek() {
                                if c == '}' {
                                    iter.next(); // consume closing brace
                                    break;
                                }
                                result.push(c);
                                iter.next();
                            }
                        }
                        continue;
                    }

                    match lookup_symbol(&name) {
                        Some(repl) => result.push_str(repl),
                        None => {
                            result.push('\\');
                            result.push_str(&name);
                        }
                    }
                }
                _ => result.push('\\'),
            }
        } else if ch == '&' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn lookup_symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        // Differential operators
        "mathrm" => return Some(""), // handled above, but fallback
        "dx" => "dx",
        "dy" => "dy",
        "dt" => "dt",

        // Greek — lowercase
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "varepsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "θ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "varpi" => "π",
        "rho" => "ρ",
        "varrho" => "ρ",
        "sigma" => "σ",
        "varsigma" => "σ",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "varphi" => "φ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",

        // Greek — uppercase
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",

        // Operators — longest first to avoid shadowing
        "infty" => "∞",
        "int" => "∫",
        "in" => "∈",
        "sum" => "∑",
        "prod" => "∏",
        "coprod" => "∐",
        "bigcap" => "⋂",
        "bigcup" => "⋃",
        "bigoplus" => "⊕",
        "bigotimes" => "⊗",
        "bigodot" => "⊙",

        // Binary operators
        "pm" => "±",
        "mp" => "∓",
        "times" => "×",
        "cdot" => "·",
        "ast" => "∗",
        "star" => "⋆",
        "circ" => "∘",
        "bullet" => "•",
        "diamond" => "⋄",
        "oplus" => "⊕",
        "ominus" => "⊖",
        "otimes" => "⊗",
        "odot" => "⊙",

        // Relations
        "leq" => "≤",
        "geq" => "≥",
        "ll" => "≪",
        "gg" => "≫",
        "prec" => "≺",
        "succ" => "≻",
        "preceq" => "≼",
        "succeq" => "≽",
        "equiv" => "≡",
        "sim" => "∼",
        "simeq" => "≃",
        "cong" => "≅",
        "approx" => "≈",
        "subset" => "⊂",
        "subseteq" => "⊆",
        "supset" => "⊃",
        "supseteq" => "⊇",
        "notin" => "∉",
        "neq" => "≠",
        "perp" => "⊥",
        "parallel" => "∥",
        "mid" => "|",
        "models" => "⊨",
        "propto" => "∝",

        // Arrows
        "longrightarrow" => "⟶",
        "longleftarrow" => "⟵",
        "Longrightarrow" => "⟹",
        "Longleftarrow" => "⟸",
        "leftrightarrow" => "↔",
        "Leftrightarrow" => "⇔",
        "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "uparrow" => "↑",
        "downarrow" => "↓",
        "Uparrow" => "⇑",
        "Downarrow" => "⇓",
        "updownarrow" => "↕",
        "nearrow" => "↗",
        "searrow" => "↘",
        "swarrow" => "↙",
        "nwarrow" => "↖",
        "mapsto" => "↦",
        "to" => "→",

        // Special
        "aleph" => "ℵ",
        "hbar" => "ℏ",
        "ell" => "ℓ",
        "nabla" => "∇",
        "partial" => "∂",
        "angle" => "∠",
        "emptyset" => "∅",
        "forall" => "∀",
        "exists" => "∃",
        "neg" => "¬",
        "land" => "∧",
        "lor" => "∨",
        "top" => "⊤",
        "bot" => "⊥",

        // Delimiters (remove)
        "left" => "",
        "right" => "",
        "big" => "",
        "Big" => "",
        "bigg" => "",
        "Bigg" => "",

        // Quad spacing
        "quad" => "  ",
        "qquad" => "    ",

        // Additional integrals
        "oint" => "∮",
        "iint" => "∬",
        "iiint" => "∭",
        "iiiint" => "⨌",
        "idotsint" => "∫⋯∫",
        "oiint" => "∯",
        "oiiint" => "∰",
        "varointclockwise" => "∲",
        "ointctrclockwise" => "∳",
        "fint" => "⨏",
        "sqint" => "⨖",
        "landupint" => "⨑",
        "lownint" => "⨕",
        "sumint" => "⨋",
        "intbar" => "⨍",
        "intBar" => "⨎",

        // More sums and products
        "bigsqcup" => "⨆",
        "biguplus" => "⨄",
        "bigsqcap" => "⨅",
        "bigvee" => "⋁",
        "bigwedge" => "⋀",
        "varprod" => "∏",

        // Limits and functions
        "limsup" => "lim sup",
        "liminf" => "lim inf",
        "arg" => "arg",
        "gcd" => "gcd",
        "hom" => "hom",
        "ker" => "ker",
        "Pr" => "Pr",
        "projlim" => "proj lim",
        "injlim" => "inj lim",
        "varlimsup" => "lim sup",
        "varliminf" => "lim inf",
        "varinjlim" => "inj lim",
        "varprojlim" => "proj lim",

        // More binary operators
        "setminus" => "∖",
        "smallsetminus" => "∖",
        "Cap" => "⋒",
        "Cup" => "⋓",
        "uplus" => "⊎",
        "sqcap" => "⊓",
        "sqcup" => "⊔",
        "doublecap" => "⋒",
        "doublecup" => "⋓",
        "dotplus" => "∔",
        "ltimes" => "⋉",
        "rtimes" => "⋊",
        "leftthreetimes" => "⋋",
        "rightthreetimes" => "⋌",
        "curlywedge" => "⋏",
        "curlyvee" => "⋎",
        "doublebarwedge" => "⩞",
        "barwedge" => "⊼",
        "veebar" => "⊻",
        "wedgebar" => "⩟",
        "boxplus" => "⊞",
        "boxminus" => "⊟",
        "boxtimes" => "⊠",
        "boxdot" => "⊡",
        "divideontimes" => "⋇",
        "lessdot" => "⋖",
        "gtrdot" => "⋗",
        "intercal" => "⊺",
        "centerdot" => "⋅",
        "smallcirc" => "∘",
        "wr" => "≀",
        "amalg" => "⨿",
        "pitchfork" => "⋔",
        "therefore" => "∴",
        "because" => "∵",

        // More relations
        "sqsubset" => "⊏",
        "sqsupset" => "⊐",
        "sqsubseteq" => "⊑",
        "sqsupseteq" => "⊒",
        "subsetneq" => "⊊",
        "supsetneq" => "⊋",
        "subsetneqq" => "⫋",
        "supsetneqq" => "⫌",
        "varsubsetneq" => "⊊",
        "varsupsetneq" => "⊋",
        "varsubsetneqq" => "⫋",
        "varsupsetneqq" => "⫌",
        "nsubseteq" => "⊈",
        "nsupseteq" => "⊉",
        "nsubseteqq" => "⊈",
        "nsupseteqq" => "⊉",
        "subseteqq" => "⊆",
        "supseteqq" => "⊇",
        "lhd" => "⊲",
        "rhd" => "⊳",
        "unlhd" => "⊴",
        "unrhd" => "⊵",
        "triangleleft" => "◁",
        "triangleright" => "▷",
        "trianglelefteq" => "⊴",
        "trianglerighteq" => "⊵",
        "vartriangleleft" => "⊲",
        "vartriangleright" => "⊳",
        "triangleq" => "≜",
        "circeq" => "≗",
        "risingdotseq" => "≓",
        "fallingdotseq" => "≒",
        "eqcirc" => "≖",
        "doteqdot" => "≑",
        "eqqsim" => "⩳",
        "eqsim" => "≂",
        "leqq" => "≦",
        "geqq" => "≧",
        "leqslant" => "⩽",
        "geqslant" => "⩾",
        "eqslantless" => "⪕",
        "eqslantgtr" => "⪖",
        "lesssim" => "≲",
        "gtrsim" => "≳",
        "lessapprox" => "⪅",
        "gtrapprox" => "⪆",
        "approxeq" => "≊",
        "lessgtr" => "≶",
        "gtrless" => "≷",
        "lesseqqgtr" => "⪋",
        "gtreqqless" => "⪌",
        "doteq" => "≐",
        "preccurlyeq" => "≼",
        "succcurlyeq" => "≽",
        "precsim" => "≾",
        "succsim" => "≿",
        "nprec" => "⊀",
        "nsucc" => "⊁",
        "npreceq" => "⋠",
        "nsucceq" => "⋡",
        "precnsim" => "⋨",
        "succnsim" => "⋩",
        "precnapprox" => "⪹",
        "succnapprox" => "⪺",
        "nless" => "≮",
        "ngtr" => "≯",
        "nleq" => "≰",
        "ngeq" => "≱",
        "lneq" => "⪇",
        "gneq" => "⪈",
        "lneqq" => "≨",
        "gneqq" => "≩",
        "lvertneqq" => "≨",
        "gvertneqq" => "≩",
        "llless" => "⋘",
        "gggtr" => "⋙",
        "lll" => "⋘",
        "ggg" => "⋙",
        "lesseqgtr" => "⋚",
        "gtreqless" => "⋛",
        "asymp" => "≍",
        "Bumpeq" => "≎",
        "bumpeq" => "≏",
        "between" => "≬",
        "backsim" => "∽",
        "backsimeq" => "⋍",
        "vDash" => "⊨",
        "Vdash" => "⊩",
        "Vvdash" => "⊪",
        "nvdash" => "⊬",
        "nvDash" => "⊭",
        "nVdash" => "⊮",
        "nVDash" => "⊯",
        "dashv" => "⊣",
        "VDash" => "⊫",

        // More arrows
        "longleftrightarrow" => "⟷",
        "Longleftrightarrow" => "⟺",
        "iff" => "⟺",
        "longmapsto" => "⟼",
        "mapsfrom" => "↤",
        "longmapsfrom" => "⟻",
        "hookrightarrow" => "↪",
        "hookleftarrow" => "↩",
        "rightharpoonup" => "⇀",
        "rightharpoondown" => "⇁",
        "leftharpoonup" => "↼",
        "leftharpoondown" => "↽",
        "upharpoonleft" => "↿",
        "upharpoonright" => "↾",
        "downharpoonleft" => "⇃",
        "downharpoonright" => "⇂",
        "rightleftharpoons" => "⇌",
        "leftrightharpoons" => "⇋",
        "rightarrowtail" => "↣",
        "leftarrowtail" => "↢",
        "twoheadrightarrow" => "↠",
        "twoheadleftarrow" => "↞",
        "Lsh" => "↰",
        "Rsh" => "↱",
        "looparrowright" => "↬",
        "looparrowleft" => "↫",
        "curvearrowright" => "↷",
        "curvearrowleft" => "↶",
        "circlearrowright" => "↺",
        "circlearrowleft" => "↻",
        "rightsquigarrow" => "⇝",
        "leftrightsquigarrow" => "↭",
        "nrightarrow" => "↛",
        "nleftarrow" => "↚",
        "nRightarrow" => "⇏",
        "nLeftarrow" => "⇍",
        "nleftrightarrow" => "↮",
        "nLeftrightarrow" => "⇎",
        "gets" => "←",
        "leadsto" => "⇝",

        // Dots
        "ldots" => "…",
        "cdots" => "⋯",
        "vdots" => "⋮",
        "ddots" => "⋱",
        "adots" => "⋰",
        "hdotsfor" => "...",
        "dotsc" => "…",
        "dotsb" => "⋯",
        "dotsm" => "⋯",
        "dotsi" => "⋯",
        "dotso" => "…",

        // Delimiters
        "lceil" => "⌈",
        "rceil" => "⌉",
        "lfloor" => "⌊",
        "rfloor" => "⌋",
        "langle" => "⟨",
        "rangle" => "⟩",
        "lmoustache" => "⎰",
        "rmoustache" => "⎱",
        "lgroup" => "︷",
        "rgroup" => "︸",
        "bracevert" => "│",
        "Arrowvert" => "∥",
        "arrowvert" => "|",
        "Vert" => "‖",
        "vert" => "|",

        // More special
        "wp" => "℘",
        "Re" => "ℜ",
        "Im" => "ℑ",
        "imath" => "ı",
        "jmath" => "ȷ",
        "beth" => "ℶ",
        "gimel" => "ℷ",
        "daleth" => "ℸ",
        "Game" => "⅁",
        "mho" => "℧",
        "Finv" => "Ⅎ",
        "eth" => "ð",
        "Bbbk" => "𝕜",
        "comp" => "∁",
        "complement" => "∁",
        "degree" => "°",
        "prime" => "′",
        "backprime" => "‵",
        "surd" => "√",
        "flat" => "♭",
        "natural" => "♮",
        "sharp" => "♯",
        "clubsuit" => "♣",
        "diamondsuit" => "♢",
        "heartsuit" => "♡",
        "spadesuit" => "♠",
        "bigstar" => "★",
        "Sun" => "☉",
        "Mercury" => "☿",
        "Venus" => "♀",
        "Earth" => "⊕",
        "Mars" => "♂",
        "Jupiter" => "♃",
        "Saturn" => "♄",
        "Uranus" => "♅",
        "Neptune" => "♆",
        "Pluto" => "♇",
        "Moon" => "☽",

        // Blackboard bold
        "BbbA" => "𝔸",
        "BbbB" => "𝔹",
        "BbbC" => "ℂ",
        "BbbD" => "𝔻",
        "BbbE" => "𝔼",
        "BbbF" => "𝔽",
        "BbbG" => "𝔾",
        "BbbH" => "ℍ",
        "BbbI" => "𝕀",
        "BbbJ" => "𝕁",
        "BbbK" => "𝕂",
        "BbbL" => "𝕃",
        "BbbM" => "𝕄",
        "BbbN" => "ℕ",
        "BbbO" => "𝕆",
        "BbbP" => "ℙ",
        "BbbQ" => "ℚ",
        "BbbR" => "ℝ",
        "BbbS" => "𝕊",
        "BbbT" => "𝕋",
        "BbbU" => "𝕌",
        "BbbV" => "𝕍",
        "BbbW" => "𝕎",
        "BbbX" => "𝕏",
        "BbbY" => "𝕐",
        "BbbZ" => "ℤ",
        "Bbb1" => "𝟙",

        // Script / calligraphic
        "scrA" => "𝒜",
        "scrB" => "ℬ",
        "scrC" => "𝒞",
        "scrD" => "𝒟",
        "scrE" => "ℰ",
        "scrF" => "ℱ",
        "scrG" => "𝒢",
        "scrH" => "ℋ",
        "scrI" => "ℐ",
        "scrJ" => "𝒥",
        "scrK" => "𝒦",
        "scrL" => "ℒ",
        "scrM" => "ℳ",
        "scrN" => "𝒩",
        "scrO" => "𝒪",
        "scrP" => "𝒫",
        "scrQ" => "𝒬",
        "scrR" => "ℛ",
        "scrS" => "𝒮",
        "scrT" => "𝒯",
        "scrU" => "𝒰",
        "scrV" => "𝒱",
        "scrW" => "𝒲",
        "scrX" => "𝒳",
        "scrY" => "𝒴",
        "scrZ" => "𝒵",

        // Fraktur
        "frakA" => "𝔄",
        "frakB" => "𝔅",
        "frakC" => "ℭ",
        "frakD" => "𝔇",
        "frakE" => "𝔈",
        "frakF" => "𝔉",
        "frakG" => "𝔊",
        "frakH" => "ℌ",
        "frakI" => "ℑ",
        "frakJ" => "𝔍",
        "frakK" => "𝔎",
        "frakL" => "𝔏",
        "frakM" => "𝔐",
        "frakN" => "𝔑",
        "frakO" => "𝔒",
        "frakP" => "𝔓",
        "frakQ" => "𝔔",
        "frakR" => "ℜ",
        "frakS" => "𝔖",
        "frakT" => "𝔗",
        "frakU" => "𝔘",
        "frakV" => "𝔙",
        "frakW" => "𝔚",
        "frakX" => "𝔛",
        "frakY" => "𝔜",
        "frakZ" => "ℨ",

        // Geometry
        "triangle" => "△",
        "bigtriangleup" => "△",
        "bigtriangledown" => "▽",
        "square" => "□",
        "blacksquare" => "■",
        "lozenge" => "◊",
        "blacklozenge" => "◆",
        "bigcirc" => "◯",
        "measuredangle" => "∡",
        "sphericalangle" => "∢",
        "triangledown" => "▽",

        // Circled operators
        "circledast" => "⊛",
        "circledcirc" => "⊚",
        "circleddash" => "⊝",
        "circledR" => "®",
        "circledS" => "Ⓢ",

        // Font style commands that strip content
        "mathscr" => "",
        "mathnormal" => "",

        // Spacing commands
        "thinspace" => "",
        "medspace" => "",
        "thickspace" => "",
        "negthinspace" => "",
        "negmedspace" => "",
        "neghthickspace" => "",
        "enspace" => "",
        "emsp" => "",
        "enskip" => "",
        "hspace" => "",
        "vspace" => "",
        "kern" => "",
        "mskip" => "",
        "mkern" => "",
        "phantom" => "",
        "hphantom" => "",
        "vphantom" => "",
        "smash" => "",

        // Math style switches — no-op in Unicode output
        "displaystyle" => "",
        "textstyle" => "",
        "scriptstyle" => "",
        "scriptscriptstyle" => "",

        // Over/under commands (accents handled by format_accents in math_formatter)
        "overbrace" => "",
        "underbrace" => "",
        "overset" => "",
        "underset" => "",
        "stackrel" => "",

        // Fractions and roots
        "frac" => "",
        "dfrac" => "",
        "tfrac" => "",
        "cfrac" => "",
        "genfrac" => "",
        "binom" => "",
        "tbinom" => "",
        "dbinom" => "",

        // Matrix environments
        "matrix" => "",
        "pmatrix" => "",
        "bmatrix" => "",
        "Bmatrix" => "",
        "vmatrix" => "",
        "Vmatrix" => "",
        "smallmatrix" => "",

        // Function names (typeset in upright text)
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "cot" => "cot",
        "sec" => "sec",
        "csc" => "csc",
        "arcsin" => "arcsin",
        "arccos" => "arccos",
        "arctan" => "arctan",
        "sinh" => "sinh",
        "cosh" => "cosh",
        "tanh" => "tanh",
        "coth" => "coth",
        "log" => "log",
        "ln" => "ln",
        "lg" => "lg",
        "exp" => "exp",
        "lim" => "lim",
        "min" => "min",
        "max" => "max",
        "sup" => "sup",
        "inf" => "inf",
        "det" => "det",
        "dim" => "dim",
        "deg" => "deg",

        // Common aliases
        "le" => "≤",
        "ge" => "≥",
        "ne" => "≠",
        "dots" => "…",
        "empty" => "∅",
        "varnothing" => "∅",
        "lnot" => "¬",

        // Negated relations
        "ncong" => "≇",
        "nsim" => "≁",
        "napprox" => "≉",
        "nasymp" => "≭",
        "nmid" => "∤",
        "nparallel" => "∦",
        "nshortmid" => "∤",
        "nshortparallel" => "∦",
        "nsqsubset" => "⋢",
        "nsqsupset" => "⋣",
        "nsqsubseteq" => "⋤",
        "nsqsupseteq" => "⋥",
        "ntriangleleft" => "⋪",
        "ntriangleright" => "⋫",
        "ntrianglelefteq" => "⋬",
        "ntrianglerighteq" => "⋭",
        "nexists" => "∄",
        "nsubseteqq" => "⊈",
        "nsupseteqq" => "⊉",

        // Misc missing symbols
        "Join" => "⋈",
        "smile" => "⌣",
        "frown" => "⌢",
        "smallsmile" => "⌣",
        "smallfrown" => "⌢",
        "coloneq" => "≔",
        "eqcolon" => "≕",
        "coloneqq" => "≕",
        "shortmid" => "∣",
        "shortparallel" => "∥",
        "bigtimes" => "⨯",
        "varpropto" => "∝",
        "digamma" => "ϝ",
        "Digamma" => "Ϝ",
        "backepsilon" => "϶",
        "Epsilon" => "Ε",

        // Additional negated relations (not already defined above)
        "nleqslant" => "≰",
        "ngeqslant" => "≱",
        "nleqq" => "≦",
        "ngeqq" => "≧",
        "nsubset" => "⊄",
        "nsupset" => "⊅",

        // Additional arrows (not already defined above)
        "dashrightarrow" => "⇢",
        "dashleftarrow" => "⇠",
        "dasharrow" => "⇢",
        "multimap" => "⊸",
        "upuparrows" => "⇈",
        "downdownarrows" => "⇊",
        "twoheadmapsto" => "⤤",
        "leftsquigarrow" => "⇜",
        "xrightarrow" => "→",
        "xleftarrow" => "←",
        "xRightarrow" => "⇒",
        "xLeftarrow" => "⇐",
        "xleftrightarrow" => "↔",
        "xLeftrightarrow" => "⇔",

        // Additional operators (not already defined above)
        "dotminus" => "∸",
        "ldotp" => ".",
        "cdotp" => "·",
        "bmod" => "mod",
        "mod" => "mod",
        "pod" => "mod",

        // Additional special symbols (not already defined above)
        "checkmark" => "✓",
        "ballotbox" => "☐",
        "maltese" => "✠",

        // Sized delimiters
        "bigl" => "",
        "bigr" => "",
        "Bigl" => "",
        "Bigr" => "",
        "biggl" => "",
        "biggr" => "",
        "Biggl" => "",
        "Biggr" => "",

        // Additional function names (not already defined above)
        "arcsec" => "arcsec",
        "arccsc" => "arccsc",
        "arccot" => "arccot",
        "arcsinh" => "arcsinh",
        "arccosh" => "arccosh",
        "arctanh" => "arctanh",

        // Subset/superset variants (not already defined above)
        "Subset" => "⋐",
        "Supset" => "⋑",

        // Additional geometry
        "vartriangle" => "△",
        "blacktriangle" => "▲",
        "blacktriangledown" => "▼",
        "Box" => "□",
        "Diamond" => "◇",

        // Negation prefix and placement control
        "not" => "¬",
        "limits" => "",
        "nolimits" => "",
        "displaylimits" => "",

        // Math lap commands (smash overlap)
        "mathclap" => "",
        "mathllap" => "",
        "mathrlap" => "",

        // Boxed and substack (handled in MathML dispatch, no-op in Unicode)
        "boxed" => "",
        "substack" => "",

        // Row separator in matrices
        "cr" => "",

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::replace_math_symbols;

    #[test]
    fn test_greek_letters() {
        assert_eq!(replace_math_symbols("\\alpha"), "α");
        assert_eq!(replace_math_symbols("\\beta"), "β");
        assert_eq!(replace_math_symbols("\\Gamma"), "Γ");
        assert_eq!(replace_math_symbols("\\Delta"), "Δ");
    }

    #[test]
    fn test_operators() {
        assert_eq!(replace_math_symbols("\\int"), "∫");
        assert_eq!(replace_math_symbols("\\sum"), "∑");
        assert_eq!(replace_math_symbols("\\infty"), "∞");
        assert_eq!(replace_math_symbols("\\pm"), "±");
        assert_eq!(replace_math_symbols("\\times"), "×");
    }

    #[test]
    fn test_relations() {
        assert_eq!(replace_math_symbols("\\leq"), "≤");
        assert_eq!(replace_math_symbols("\\geq"), "≥");
        assert_eq!(replace_math_symbols("\\neq"), "≠");
        assert_eq!(replace_math_symbols("\\in"), "∈");
    }

    #[test]
    fn test_arrows() {
        assert_eq!(replace_math_symbols("\\rightarrow"), "→");
        assert_eq!(replace_math_symbols("\\Leftarrow"), "⇐");
        assert_eq!(replace_math_symbols("\\to"), "→");
    }

    #[test]
    fn test_spacing_removed() {
        assert_eq!(replace_math_symbols("a\\,b"), "a b");
        assert_eq!(replace_math_symbols("a\\;b"), "a b");
        assert_eq!(replace_math_symbols("a\\!b"), "ab");
    }

    #[test]
    fn test_multiple_symbols() {
        let result = replace_math_symbols("\\alpha + \\beta = \\gamma");
        assert_eq!(result, "α + β = γ");
    }

    #[test]
    fn test_infinity_before_in() {
        // \infty must be replaced before \in to avoid partial matches
        assert_eq!(replace_math_symbols("\\infty"), "∞");
    }

    #[test]
    fn test_additional_integrals() {
        assert_eq!(replace_math_symbols("\\oint"), "∮");
        assert_eq!(replace_math_symbols("\\iint"), "∬");
        assert_eq!(replace_math_symbols("\\iiint"), "∭");
        assert_eq!(replace_math_symbols("\\oiint"), "∯");
    }

    #[test]
    fn test_more_arrows() {
        assert_eq!(replace_math_symbols("\\iff"), "⟺");
        assert_eq!(replace_math_symbols("\\hookrightarrow"), "↪");
        assert_eq!(replace_math_symbols("\\rightleftharpoons"), "⇌");
        assert_eq!(replace_math_symbols("\\longmapsto"), "⟼");
    }

    #[test]
    fn test_delimiters() {
        assert_eq!(replace_math_symbols("\\lceil"), "⌈");
        assert_eq!(replace_math_symbols("\\rceil"), "⌉");
        assert_eq!(replace_math_symbols("\\lfloor"), "⌊");
        assert_eq!(replace_math_symbols("\\rfloor"), "⌋");
        assert_eq!(replace_math_symbols("\\langle"), "⟨");
        assert_eq!(replace_math_symbols("\\rangle"), "⟩");
    }

    #[test]
    fn test_blackboard_bold() {
        assert_eq!(replace_math_symbols("\\BbbR"), "ℝ");
        assert_eq!(replace_math_symbols("\\BbbC"), "ℂ");
        assert_eq!(replace_math_symbols("\\BbbN"), "ℕ");
        assert_eq!(replace_math_symbols("\\BbbZ"), "ℤ");
        assert_eq!(replace_math_symbols("\\BbbQ"), "ℚ");
    }

    #[test]
    fn test_script_letters() {
        assert_eq!(replace_math_symbols("\\scrL"), "ℒ");
        assert_eq!(replace_math_symbols("\\scrH"), "ℋ");
        assert_eq!(replace_math_symbols("\\scrR"), "ℛ");
    }

    #[test]
    fn test_more_relations() {
        assert_eq!(replace_math_symbols("\\subsetneq"), "⊊");
        assert_eq!(replace_math_symbols("\\supsetneq"), "⊋");
        assert_eq!(replace_math_symbols("\\nleq"), "≰");
        assert_eq!(replace_math_symbols("\\ngeq"), "≱");
        assert_eq!(replace_math_symbols("\\approxeq"), "≊");
    }

    #[test]
    fn test_dots() {
        assert_eq!(replace_math_symbols("\\ldots"), "…");
        assert_eq!(replace_math_symbols("\\cdots"), "⋯");
        assert_eq!(replace_math_symbols("\\vdots"), "⋮");
        assert_eq!(replace_math_symbols("\\ddots"), "⋱");
    }

    #[test]
    fn test_geometry() {
        assert_eq!(replace_math_symbols("\\square"), "□");
        assert_eq!(replace_math_symbols("\\triangle"), "△");
        assert_eq!(replace_math_symbols("\\angle"), "∠");
        assert_eq!(replace_math_symbols("\\measuredangle"), "∡");
    }

    #[test]
    fn test_font_commands_strip_braces() {
        // text and mathscr still strip braces; math alphabets now pass through for later processing
        assert_eq!(replace_math_symbols("\\mathscr{A}"), "A");
        assert_eq!(replace_math_symbols("\\text{R}"), "R");
        // Math alphabets are preserved so format_math_alphabets can transform them
        assert_eq!(replace_math_symbols("\\mathbb{R}"), "\\mathbb{R}");
        assert_eq!(replace_math_symbols("\\mathfrak{g}"), "\\mathfrak{g}");
    }

    #[test]
    fn test_function_names() {
        assert_eq!(replace_math_symbols("\\sin"), "sin");
        assert_eq!(replace_math_symbols("\\cos"), "cos");
        assert_eq!(replace_math_symbols("\\tan"), "tan");
        assert_eq!(replace_math_symbols("\\log"), "log");
        assert_eq!(replace_math_symbols("\\ln"), "ln");
        assert_eq!(replace_math_symbols("\\lim"), "lim");
        assert_eq!(replace_math_symbols("\\max"), "max");
        assert_eq!(replace_math_symbols("\\min"), "min");
        assert_eq!(replace_math_symbols("\\arctan"), "arctan");
        assert_eq!(replace_math_symbols("\\det"), "det");
    }

    #[test]
    fn test_negated_relations() {
        assert_eq!(replace_math_symbols("\\ncong"), "≇");
        assert_eq!(replace_math_symbols("\\nsim"), "≁");
        assert_eq!(replace_math_symbols("\\napprox"), "≉");
        assert_eq!(replace_math_symbols("\\nmid"), "∤");
        assert_eq!(replace_math_symbols("\\nparallel"), "∦");
        assert_eq!(replace_math_symbols("\\nexists"), "∄");
        assert_eq!(replace_math_symbols("\\ntriangleleft"), "⋪");
        assert_eq!(replace_math_symbols("\\ntrianglerighteq"), "⋭");
    }

    #[test]
    fn test_common_aliases() {
        assert_eq!(replace_math_symbols("\\le"), "≤");
        assert_eq!(replace_math_symbols("\\ge"), "≥");
        assert_eq!(replace_math_symbols("\\ne"), "≠");
        assert_eq!(replace_math_symbols("\\dots"), "…");
        assert_eq!(replace_math_symbols("\\varnothing"), "∅");
        assert_eq!(replace_math_symbols("\\lnot"), "¬");
    }

    #[test]
    fn test_misc_symbols() {
        assert_eq!(replace_math_symbols("\\Join"), "⋈");
        assert_eq!(replace_math_symbols("\\smile"), "⌣");
        assert_eq!(replace_math_symbols("\\frown"), "⌢");
        assert_eq!(replace_math_symbols("\\coloneq"), "≔");
        assert_eq!(replace_math_symbols("\\bigtimes"), "⨯");
        assert_eq!(replace_math_symbols("\\digamma"), "ϝ");
        assert_eq!(replace_math_symbols("\\backepsilon"), "϶");
    }

    #[test]
    fn test_boldsymbol_strips_braces() {
        assert_eq!(replace_math_symbols("\\boldsymbol{x}"), "x");
        // \boldsymbol strips command + braces, inner content is raw
        assert_eq!(replace_math_symbols("\\boldsymbol{\\alpha}"), "\\alpha");
    }

    #[test]
    fn test_pmb_strips_braces() {
        assert_eq!(replace_math_symbols("\\pmb{x}"), "x");
    }

    #[test]
    fn test_operatorname_strips_braces() {
        assert_eq!(replace_math_symbols("\\operatorname{Tr}"), "Tr");
    }

    #[test]
    fn test_displaystyle_noop() {
        assert_eq!(replace_math_symbols("\\displaystyle"), "");
        assert_eq!(replace_math_symbols("\\textstyle"), "");
        assert_eq!(replace_math_symbols("\\scriptstyle"), "");
        assert_eq!(replace_math_symbols("\\scriptscriptstyle"), "");
    }

    #[test]
    fn test_additional_negated_relations() {
        assert_eq!(replace_math_symbols("\\nleqslant"), "≰");
        assert_eq!(replace_math_symbols("\\ngeqslant"), "≱");
        assert_eq!(replace_math_symbols("\\nleqq"), "≦");
        assert_eq!(replace_math_symbols("\\ngeqq"), "≧");
        assert_eq!(replace_math_symbols("\\nsubset"), "⊄");
        assert_eq!(replace_math_symbols("\\nsupset"), "⊅");
    }

    #[test]
    fn test_additional_arrows() {
        assert_eq!(replace_math_symbols("\\dashrightarrow"), "⇢");
        assert_eq!(replace_math_symbols("\\dashleftarrow"), "⇠");
        assert_eq!(replace_math_symbols("\\multimap"), "⊸");
        assert_eq!(replace_math_symbols("\\upuparrows"), "⇈");
        assert_eq!(replace_math_symbols("\\downdownarrows"), "⇊");
        assert_eq!(replace_math_symbols("\\leftsquigarrow"), "⇜");
    }

    #[test]
    fn test_additional_operators() {
        assert_eq!(replace_math_symbols("\\dotminus"), "∸");
        assert_eq!(replace_math_symbols("\\ldotp"), ".");
        assert_eq!(replace_math_symbols("\\cdotp"), "·");
        assert_eq!(replace_math_symbols("\\bmod"), "mod");
        assert_eq!(replace_math_symbols("\\mod"), "mod");
        assert_eq!(replace_math_symbols("\\pod"), "mod");
    }

    #[test]
    fn test_additional_special_symbols() {
        assert_eq!(replace_math_symbols("\\checkmark"), "✓");
        assert_eq!(replace_math_symbols("\\ballotbox"), "☐");
        assert_eq!(replace_math_symbols("\\maltese"), "✠");
    }

    #[test]
    fn test_sized_delimiters() {
        assert_eq!(replace_math_symbols("\\bigl"), "");
        assert_eq!(replace_math_symbols("\\bigr"), "");
        assert_eq!(replace_math_symbols("\\Bigl"), "");
        assert_eq!(replace_math_symbols("\\Biggr"), "");
    }

    #[test]
    fn test_additional_function_names() {
        assert_eq!(replace_math_symbols("\\arcsec"), "arcsec");
        assert_eq!(replace_math_symbols("\\arccsc"), "arccsc");
        assert_eq!(replace_math_symbols("\\arccot"), "arccot");
        assert_eq!(replace_math_symbols("\\arcsinh"), "arcsinh");
        assert_eq!(replace_math_symbols("\\arccosh"), "arccosh");
        assert_eq!(replace_math_symbols("\\arctanh"), "arctanh");
    }

    #[test]
    fn test_subset_superset_variants() {
        assert_eq!(replace_math_symbols("\\Subset"), "⋐");
        assert_eq!(replace_math_symbols("\\Supset"), "⋑");
    }

    #[test]
    fn test_additional_geometry() {
        assert_eq!(replace_math_symbols("\\vartriangle"), "△");
        assert_eq!(replace_math_symbols("\\blacktriangle"), "▲");
        assert_eq!(replace_math_symbols("\\blacktriangledown"), "▼");
        assert_eq!(replace_math_symbols("\\Box"), "□");
        assert_eq!(replace_math_symbols("\\Diamond"), "◇");
    }

    #[test]
    fn test_xarrow_aliases() {
        assert_eq!(replace_math_symbols("\\xrightarrow"), "→");
        assert_eq!(replace_math_symbols("\\xleftarrow"), "←");
        assert_eq!(replace_math_symbols("\\xRightarrow"), "⇒");
        assert_eq!(replace_math_symbols("\\xLeftarrow"), "⇐");
    }

    #[test]
    fn test_not_and_limits() {
        assert_eq!(replace_math_symbols("\\not"), "¬");
        assert_eq!(replace_math_symbols("\\limits"), "");
        assert_eq!(replace_math_symbols("\\nolimits"), "");
        assert_eq!(replace_math_symbols("\\displaylimits"), "");
    }

    #[test]
    fn test_math_lap_commands() {
        assert_eq!(replace_math_symbols("\\mathclap"), "");
        assert_eq!(replace_math_symbols("\\mathllap"), "");
        assert_eq!(replace_math_symbols("\\mathrlap"), "");
    }

    #[test]
    fn test_boxed_substack_cr() {
        assert_eq!(replace_math_symbols("\\boxed"), "");
        assert_eq!(replace_math_symbols("\\substack"), "");
        assert_eq!(replace_math_symbols("\\cr"), "");
    }
}
