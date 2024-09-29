#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VcsType {
    Bazaar,
    Git,
    Hg,
    Svn,
    Fossil,
    Darcs,
    Cvs,
    Arch,
    Svk
}
