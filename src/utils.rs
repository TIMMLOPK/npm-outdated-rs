pub fn characters_diff<'a>(string1: &'a str, string2: &'a str) -> Vec<&'a str> {
    let mut diff = Vec::new();
    let mut chars1 = string1.chars();
    let mut chars2 = string2.chars();
    let mut char1 = chars1.next();
    let mut char2 = chars2.next();
    while char1 != None && char2 != None && char1 == char2 {
        char1 = chars1.next();
        char2 = chars2.next();
    }
    if char1 != None {
        diff.push(&string1[string1.len() - chars1.as_str().len()..]);
    }
    if char2 != None {
        diff.push(&string2[string2.len() - chars2.as_str().len()..]);
    }
    diff
}