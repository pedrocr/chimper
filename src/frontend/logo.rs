extern crate rand;
use self::rand::Rng;

pub fn random() -> &'static [u8] {
  let mut logos = Vec::<&'static [u8]>::new();
  logos.push(include_bytes!("../../icons/chimp1.svg.png"));
  logos.push(include_bytes!("../../icons/chimp2.svg.png"));
  logos.push(include_bytes!("../../icons/chimp3.svg.png"));
  logos.push(include_bytes!("../../icons/chimp4.svg.png"));
  logos.push(include_bytes!("../../icons/chimp5.svg.png"));
  logos.push(include_bytes!("../../icons/chimp6.svg.png"));
  logos.push(include_bytes!("../../icons/chimp7.svg.png"));
  logos.push(include_bytes!("../../icons/chimp8.svg.png"));
  logos.push(include_bytes!("../../icons/chimp9.svg.png"));

  let mut rng = rand::thread_rng();
  let idx = rng.gen_range(0..logos.len());
  logos[idx]
}
