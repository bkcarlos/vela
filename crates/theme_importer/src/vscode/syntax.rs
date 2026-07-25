use indexmap::IndexMap;
use serde::Deserialize;
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum VsCodeTokenScope {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct VsCodeTokenColor {
    pub name: Option<String>,
    pub scope: Option<VsCodeTokenScope>,
    pub settings: VsCodeTokenColorSettings,
}

#[derive(Debug, Deserialize)]
pub struct VsCodeTokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,
}

#[derive(Debug, PartialEq, Copy, Clone, EnumIter)]
pub enum VelaSyntaxToken {
    Attribute,
    Boolean,
    Comment,
    CommentDoc,
    Constant,
    Constructor,
    Embedded,
    Emphasis,
    EmphasisStrong,
    Enum,
    Function,
    Hint,
    Keyword,
    Label,
    LinkText,
    LinkUri,
    Number,
    Operator,
    Predictive,
    Preproc,
    Primary,
    Property,
    Punctuation,
    PunctuationBracket,
    PunctuationDelimiter,
    PunctuationListMarker,
    PunctuationSpecial,
    String,
    StringEscape,
    StringRegex,
    StringSpecial,
    StringSpecialSymbol,
    Tag,
    TextLiteral,
    Title,
    Type,
    Variable,
    VariableSpecial,
    Variant,
}

impl std::fmt::Display for VelaSyntaxToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VelaSyntaxToken::Attribute => "attribute",
                VelaSyntaxToken::Boolean => "boolean",
                VelaSyntaxToken::Comment => "comment",
                VelaSyntaxToken::CommentDoc => "comment.doc",
                VelaSyntaxToken::Constant => "constant",
                VelaSyntaxToken::Constructor => "constructor",
                VelaSyntaxToken::Embedded => "embedded",
                VelaSyntaxToken::Emphasis => "emphasis",
                VelaSyntaxToken::EmphasisStrong => "emphasis.strong",
                VelaSyntaxToken::Enum => "enum",
                VelaSyntaxToken::Function => "function",
                VelaSyntaxToken::Hint => "hint",
                VelaSyntaxToken::Keyword => "keyword",
                VelaSyntaxToken::Label => "label",
                VelaSyntaxToken::LinkText => "link_text",
                VelaSyntaxToken::LinkUri => "link_uri",
                VelaSyntaxToken::Number => "number",
                VelaSyntaxToken::Operator => "operator",
                VelaSyntaxToken::Predictive => "predictive",
                VelaSyntaxToken::Preproc => "preproc",
                VelaSyntaxToken::Primary => "primary",
                VelaSyntaxToken::Property => "property",
                VelaSyntaxToken::Punctuation => "punctuation",
                VelaSyntaxToken::PunctuationBracket => "punctuation.bracket",
                VelaSyntaxToken::PunctuationDelimiter => "punctuation.delimiter",
                VelaSyntaxToken::PunctuationListMarker => "punctuation.list_marker",
                VelaSyntaxToken::PunctuationSpecial => "punctuation.special",
                VelaSyntaxToken::String => "string",
                VelaSyntaxToken::StringEscape => "string.escape",
                VelaSyntaxToken::StringRegex => "string.regex",
                VelaSyntaxToken::StringSpecial => "string.special",
                VelaSyntaxToken::StringSpecialSymbol => "string.special.symbol",
                VelaSyntaxToken::Tag => "tag",
                VelaSyntaxToken::TextLiteral => "text.literal",
                VelaSyntaxToken::Title => "title",
                VelaSyntaxToken::Type => "type",
                VelaSyntaxToken::Variable => "variable",
                VelaSyntaxToken::VariableSpecial => "variable.special",
                VelaSyntaxToken::Variant => "variant",
            }
        )
    }
}

impl VelaSyntaxToken {
    pub fn find_best_token_color_match<'a>(
        &self,
        token_colors: &'a [VsCodeTokenColor],
    ) -> Option<&'a VsCodeTokenColor> {
        let mut ranked_matches = IndexMap::new();

        for (ix, token_color) in token_colors.iter().enumerate() {
            if token_color.settings.foreground.is_none() {
                continue;
            }

            let Some(rank) = self.rank_match(token_color) else {
                continue;
            };

            if rank > 0 {
                ranked_matches.insert(ix, rank);
            }
        }

        ranked_matches
            .into_iter()
            .max_by_key(|(_, rank)| *rank)
            .map(|(ix, _)| &token_colors[ix])
    }

    fn rank_match(&self, token_color: &VsCodeTokenColor) -> Option<u32> {
        let candidate_scopes = match token_color.scope.as_ref()? {
            VsCodeTokenScope::One(scope) => vec![scope],
            VsCodeTokenScope::Many(scopes) => scopes.iter().collect(),
        }
        .iter()
        .flat_map(|scope| scope.split(',').map(|s| s.trim()))
        .collect::<Vec<_>>();

        let scopes_to_match = self.to_vscode();
        let number_of_scopes_to_match = scopes_to_match.len();

        let mut matches = 0;

        for (ix, scope) in scopes_to_match.into_iter().enumerate() {
            // Assign each entry a weight that is inversely proportional to its
            // position in the list.
            //
            // Entries towards the front are weighted higher than those towards the end.
            let weight = (number_of_scopes_to_match - ix) as u32;

            if candidate_scopes.contains(&scope) {
                matches += 1 + weight;
            }
        }

        Some(matches)
    }

    pub fn fallbacks(&self) -> &[Self] {
        match self {
            VelaSyntaxToken::CommentDoc => &[VelaSyntaxToken::Comment],
            VelaSyntaxToken::Number => &[VelaSyntaxToken::Constant],
            VelaSyntaxToken::VariableSpecial => &[VelaSyntaxToken::Variable],
            VelaSyntaxToken::PunctuationBracket
            | VelaSyntaxToken::PunctuationDelimiter
            | VelaSyntaxToken::PunctuationListMarker
            | VelaSyntaxToken::PunctuationSpecial => &[VelaSyntaxToken::Punctuation],
            VelaSyntaxToken::StringEscape
            | VelaSyntaxToken::StringRegex
            | VelaSyntaxToken::StringSpecial
            | VelaSyntaxToken::StringSpecialSymbol => &[VelaSyntaxToken::String],
            _ => &[],
        }
    }

    fn to_vscode(self) -> Vec<&'static str> {
        match self {
            VelaSyntaxToken::Attribute => vec!["entity.other.attribute-name"],
            VelaSyntaxToken::Boolean => vec!["constant.language"],
            VelaSyntaxToken::Comment => vec!["comment"],
            VelaSyntaxToken::CommentDoc => vec!["comment.block.documentation"],
            VelaSyntaxToken::Constant => {
                vec!["constant", "constant.language", "constant.character"]
            }
            VelaSyntaxToken::Constructor => {
                vec![
                    "entity.name.tag",
                    "entity.name.function.definition.special.constructor",
                ]
            }
            VelaSyntaxToken::Embedded => vec!["meta.embedded"],
            VelaSyntaxToken::Emphasis => vec!["markup.italic"],
            VelaSyntaxToken::EmphasisStrong => vec![
                "markup.bold",
                "markup.italic markup.bold",
                "markup.bold markup.italic",
            ],
            VelaSyntaxToken::Enum => vec!["support.type.enum"],
            VelaSyntaxToken::Function => vec![
                "entity.function",
                "entity.name.function",
                "variable.function",
            ],
            VelaSyntaxToken::Hint => vec![],
            VelaSyntaxToken::Keyword => vec![
                "keyword",
                "keyword.other.fn.rust",
                "keyword.control",
                "keyword.control.fun",
                "keyword.control.class",
                "punctuation.accessor",
                "entity.name.tag",
            ],
            VelaSyntaxToken::Label => vec![
                "label",
                "entity.name",
                "entity.name.import",
                "entity.name.package",
            ],
            VelaSyntaxToken::LinkText => vec!["markup.underline.link", "string.other.link"],
            VelaSyntaxToken::LinkUri => vec!["markup.underline.link", "string.other.link"],
            VelaSyntaxToken::Number => vec!["constant.numeric", "number"],
            VelaSyntaxToken::Operator => vec!["operator", "keyword.operator"],
            VelaSyntaxToken::Predictive => vec![],
            VelaSyntaxToken::Preproc => vec![
                "preproc",
                "meta.preprocessor",
                "punctuation.definition.preprocessor",
            ],
            VelaSyntaxToken::Primary => vec![],
            VelaSyntaxToken::Property => vec![
                "variable.member",
                "support.type.property-name",
                "variable.object.property",
                "variable.other.field",
            ],
            VelaSyntaxToken::Punctuation => vec![
                "punctuation",
                "punctuation.section",
                "punctuation.accessor",
                "punctuation.separator",
                "punctuation.definition.tag",
            ],
            VelaSyntaxToken::PunctuationBracket => vec![
                "punctuation.bracket",
                "punctuation.definition.tag.begin",
                "punctuation.definition.tag.end",
            ],
            VelaSyntaxToken::PunctuationDelimiter => vec![
                "punctuation.delimiter",
                "punctuation.separator",
                "punctuation.terminator",
            ],
            VelaSyntaxToken::PunctuationListMarker => {
                vec!["markup.list punctuation.definition.list.begin"]
            }
            VelaSyntaxToken::PunctuationSpecial => vec!["punctuation.special"],
            VelaSyntaxToken::String => vec!["string"],
            VelaSyntaxToken::StringEscape => {
                vec!["string.escape", "constant.character", "constant.other"]
            }
            VelaSyntaxToken::StringRegex => vec!["string.regex"],
            VelaSyntaxToken::StringSpecial => vec!["string.special", "constant.other.symbol"],
            VelaSyntaxToken::StringSpecialSymbol => {
                vec!["string.special.symbol", "constant.other.symbol"]
            }
            VelaSyntaxToken::Tag => vec!["tag", "entity.name.tag", "meta.tag.sgml"],
            VelaSyntaxToken::TextLiteral => vec!["text.literal", "string"],
            VelaSyntaxToken::Title => vec!["title", "entity.name"],
            VelaSyntaxToken::Type => vec![
                "entity.name.type",
                "entity.name.type.primitive",
                "entity.name.type.numeric",
                "keyword.type",
                "support.type",
                "support.type.primitive",
                "support.class",
            ],
            VelaSyntaxToken::Variable => vec![
                "variable",
                "variable.language",
                "variable.member",
                "variable.parameter",
                "variable.parameter.function-call",
            ],
            VelaSyntaxToken::VariableSpecial => vec![
                "variable.special",
                "variable.member",
                "variable.annotation",
                "variable.language",
            ],
            VelaSyntaxToken::Variant => vec!["variant"],
        }
    }
}
