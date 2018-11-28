use dictionary::Access;
use errors::*;
use pdf_types::*;

type Result<T> = ::std::result::Result<T, PdfError>;

#[derive(Debug)]
pub struct Catalog {
    pub r#type: PdfName, // must be /Catalog
    pub version: Option<PdfName>,
    pub extensions: Option<Dictionary>,
    pub pages: Reference,
    pub page_labels: Option<Dictionary>,
    pub names: Option<Dictionary>,
    pub dests: Option<Dictionary>,
    pub viewer_preferences: Option<Dictionary>,
    pub page_layout: Option<PdfName>,
    pub page_mode: Option<PdfName>,
    pub outlines: Option<Dictionary>,
    pub threads: Option<Array>,
    pub open_action: Option<PdfObject>, // array or dictionary
    pub aa: Option<Dictionary>,
    pub uri: Option<Dictionary>,
    pub acro_form: Option<Dictionary>,
    pub metadata: Option<PdfObject>, // TODO: should be stream
    pub struct_tree_root: Option<Dictionary>,
    pub mark_info: Option<Dictionary>,
    pub lang: Option<PdfString>,
    pub spider_info: Option<Dictionary>,
    pub output_intents: Option<Array>,
    pub piece_info: Option<Dictionary>,
    pub oc_properties: Option<Dictionary>,
    pub perms: Option<Dictionary>,
    pub legal: Option<Dictionary>,
    pub requirements: Option<Array>,
    pub collection: Option<Dictionary>,
    pub needs_rendering: Option<bool>,
}

impl Catalog {
    pub fn new(mut dict: Dictionary) -> Result<Catalog> {
        Ok(Catalog {
            r#type: dict.need_type(PdfName::Catalog, PdfError::InvalidCatalog)?,
            version: dict.want_name(PdfName::Version),
            extensions: dict.want_dictionary(PdfName::Extensions),
            pages: dict.need_reference(PdfName::Pages, PdfError::InvalidCatalog)?,
            page_labels: dict.want_dictionary(PdfName::PageLabels),
            names: dict.want_dictionary(PdfName::Names),
            dests: dict.want_dictionary(PdfName::Dests),
            viewer_preferences: dict.want_dictionary(PdfName::ViewerPreferences),
            page_layout: dict.want_name(PdfName::PageLayout),
            page_mode: dict.want_name(PdfName::PageMode),
            outlines: dict.want_dictionary(PdfName::Outlines),
            threads: dict.want_array(PdfName::Threads),
            open_action: None,
            aa: dict.want_dictionary(PdfName::AA),
            uri: dict.want_dictionary(PdfName::URI),
            acro_form: dict.want_dictionary(PdfName::AcroForm),
            metadata: None,
            struct_tree_root: dict.want_dictionary(PdfName::StructTreeRoot),
            mark_info: dict.want_dictionary(PdfName::MarkInfo),
            lang: None,
            spider_info: dict.want_dictionary(PdfName::SpiderInfo),
            output_intents: dict.want_array(PdfName::OutputIntents),
            piece_info: dict.want_dictionary(PdfName::PieceInfo),
            oc_properties: dict.want_dictionary(PdfName::OCProperties),
            perms: dict.want_dictionary(PdfName::Perms),
            legal: dict.want_dictionary(PdfName::Legal),
            requirements: dict.want_array(PdfName::Requirements),
            collection: dict.want_dictionary(PdfName::Collection),
            needs_rendering: None,
        })
    }
}
