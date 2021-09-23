use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use std::str::FromStr;

type Word = String;
type Consonant = char;
type Vowel = char;
type Syllable = String;

#[derive(Debug)]
struct Consonants(Vec<Consonant>);
#[derive(Debug)]
struct Vowels(Vec<Vowel>);

#[derive(Debug)]
enum SyllableOrder {
    Vc,
    Cv,
    Cvv,
    Cvc,
    Ccv,
    Vcv,
    Vcc,
    Vvc,
}

impl SyllableOrder {
    pub fn generate(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        match self {
            SyllableOrder::Vc => self.vc(rng, cs, vs),
            SyllableOrder::Cv => self.cv(rng, cs, vs),
            SyllableOrder::Cvv => self.cvv(rng, cs, vs),
            SyllableOrder::Cvc => self.cvc(rng, cs, vs),
            SyllableOrder::Ccv => self.ccv(rng, cs, vs),
            SyllableOrder::Vcv => self.vcv(rng, cs, vs),
            SyllableOrder::Vcc => self.vcc(rng, cs, vs),
            SyllableOrder::Vvc => self.vvc(rng, cs, vs),
        }
    }

    fn vc(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() && (cs.contains(&'m') | cs.contains(&'n')) {
            format!("{}{}", self.v(rng, vs), self.n(rng, cs))
        } else {
            format!("{}{}", self.v(rng, vs), self.c(rng, cs))
        }
    }

    fn cv(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        format!("{}{}", self.c(rng, cs), self.v(rng, vs))
    }

    fn cvv(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.cv(rng, cs, vs), self.v(rng, vs))
        } else {
            self.cv(rng, cs, vs)
        }
    }

    fn cvc(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.cv(rng, cs, vs), self.c(rng, cs))
        } else if rng.gen() && (cs.contains(&'m') | cs.contains(&'n')) {
            format!("{}{}", self.cv(rng, cs, vs), self.n(rng, cs))
        } else {
            self.cv(rng, cs, vs)
        }
    }

    fn ccv(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.c(rng, cs), self.cv(rng, cs, vs))
        } else {
            self.cv(rng, cs, vs)
        }
    }

    fn vcv(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.vc(rng, cs, vs), self.v(rng, vs))
        } else {
            self.vc(rng, cs, vs)
        }
    }

    fn vcc(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.vc(rng, cs, vs), self.c(rng, cs))
        } else if rng.gen() && (cs.contains(&'m') | cs.contains(&'n')) {
            format!("{}{}", self.vc(rng, cs, vs), self.n(rng, vs))
        } else {
            self.vc(rng, cs, vs)
        }
    }

    fn vvc(&self, rng: &mut ThreadRng, cs: &[Consonant], vs: &[Vowel]) -> Syllable {
        if rng.gen() {
            format!("{}{}", self.v(rng, vs), self.vc(rng, cs, vs))
        } else if rng.gen() && (cs.contains(&'m') | cs.contains(&'n')) {
            format!("{}{}{}", self.v(rng, vs), self.v(rng, vs), self.n(rng, cs))
        } else {
            self.vc(rng, cs, vs)
        }
    }

    fn c(&self, rng: &mut ThreadRng, cs: &[Consonant]) -> Consonant {
        let idx: usize = rng.gen_range(0..cs.len());
        cs[idx]
    }

    fn v(&self, rng: &mut ThreadRng, vs: &[Vowel]) -> Vowel {
        let idx: usize = rng.gen_range(0..vs.len());
        vs[idx]
    }

    fn n(&self, rng: &mut ThreadRng, cs: &[Consonant]) -> Consonant {
        let cm: bool = cs.contains(&'m');
        let cn: bool = cs.contains(&'n');

        if cm && cn {
            if rng.gen() {
                'm'
            } else {
                'n'
            }
        } else if cn {
            'n'
        } else {
            'm'
        }
    }
}

impl FromStr for SyllableOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_uppercase().as_str() {
            "VC" => Self::Vc,
            "CV" => Self::Cv,
            "CVV" => Self::Cvv,
            "CVC" => Self::Cvc,
            "CCV" => Self::Ccv,
            "VCV" => Self::Vcv,
            "VCC" => Self::Vcc,
            "VVC" => Self::Vvc,
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to parse syllable order",
                )))
            }
        })
    }
}

#[derive(Debug)]
struct Generator {
    pub consonants: Vec<Consonant>,
    pub vowels: Vec<Vowel>,
    pub order: SyllableOrder,
    pub syllables: u128,
    pub count: u128,
}

impl Generator {
    pub fn generate(&self) {
        let mut rng: ThreadRng = thread_rng();

        for _ in 0..self.count {
            let mut word: Word = Word::new();
            for _ in 0..self.syllables {
                word = format!("{}{}", word, self.order.generate(&mut rng, &self.consonants, &self.vowels));
            }
            println!("{}", word);
        }
    }
}

fn main() -> Result<(), Error> {
    use jargon_args::Jargon;
    let mut j: Jargon = Jargon::from_env();

    if j.contains(["-h", "--help"]) {
        help();
    } else if j.contains(["-V", "--version"]) {
        version();
    } else {
        let moment = std::time::Instant::now();
        let gen: Generator = Generator {
            consonants: j.result_arg(["-c", "--consonants"])?.chars().collect(),
            vowels: j.result_arg(["-v", "--vowels"])?.chars().collect(),
            order: j.result_arg(["-o", "--order"])?.parse()?,
            syllables: j
                .option_arg(["-s", "--syllables"])
                .unwrap_or_else(|| "2".to_string())
                .parse()?,
            count: j
                .option_arg(["-C", "--count"])
                .unwrap_or_else(|| "4".to_string())
                .parse()?,
        };

        gen.generate();

        eprintln!("took {} millisecond(s)", moment.elapsed().as_millis());
    }

    Ok(())
}

type Error = Box<dyn std::error::Error>;

fn help() {
    print!(
        "Rawrs - Random Annoying Words

\t-c, --consonants\tList of consonants to use
\t-h, --help\t\tView this help
\t-o, --order\t\tSelect syllable order
\t-s, --syllables\t\tNumber of syllables for each word
\t-v, --vowels\t\tList of vowels to use
\t-V, --version\t\tView version information\n"
    );
}

fn version() {
    println!("Rawrs version {}", env!("CARGO_PKG_VERSION"));
}
