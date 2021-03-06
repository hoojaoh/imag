//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

///
/// Generate variants of a base value by applying parts
///
/// Example:
///
/// ```
/// use libimagutil::variants::generate_variants;
///     let base = 1;
///     let vars = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
///     let res = generate_variants(&base, vars.iter(), &|base, var| base + var);
///     assert_eq!(res, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
/// ```
///
pub fn generate_variants<A, B, C, F, I>(base: &A, modders: I, f: &F)
    -> Vec<C>
    where
        F: Fn(&A, B) -> C,
        I: Iterator<Item = B>
{
    modders.map(|m| f(base, m)).collect()
}

#[cfg(test)]
mod test {

    use super::generate_variants;

    #[test]
    fn test_variants_simple() {
        let base = 1;
        let vars = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let res = generate_variants(&base, vars.iter(), &|base, var| base + var);

        assert!(res.len() == 11, format!("Length is {} instead of 11", res.len()));
        assert!(res.iter().all(|i| *i > 0));
        assert!(res == vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    }

    #[test]
    fn test_variants_pathes() {
        use std::path::PathBuf;

        let base = PathBuf::from("/");
        let vars = vec!["foo", "bar", "baz"];
        let res = generate_variants(&base, vars.iter(), &|base, var| {
            let mut base = base.clone();
            base.push(var);
            base
        });

        assert!(res.len() == 3, format!("Length is {} instead of 3", res.len()));
        let eq_vec = vec!["/foo", "/bar", "/baz"];
        let eq     = eq_vec.iter().map(PathBuf::from);
        assert!(res.into_iter().zip(eq).all(|(orig, equi)| orig == equi));
    }

}

