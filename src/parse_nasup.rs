pub mod parse_model;
pub mod parse_presenter_institutions;
pub mod parse_sessions;
pub mod parse_strands;

fn find_commas_without_following_whitespace(text: &str) -> Vec<usize> {
  text
    .char_indices()
    .zip(text.chars().skip(1))
    .filter_map(|((i, c), next)| {
      if c == ',' && !next.is_whitespace() {
        Some(i)
      } else {
        None
      }
    })
    .collect()
}
