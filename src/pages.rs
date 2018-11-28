use dictionary::{ExtractOption, ExtractRequired};
use errors::*;
use pdf_types::*;

type Result<T> = ::std::result::Result<T, PdfError>;

#[derive(Debug)]
pub struct Pages {
    pub r#type: PdfName, // must be /Pages
    pub parent: Option<Reference>,
    pub kids: Array,
    pub count: u32,
}

impl Pages {
    pub fn new(mut dict: Dictionary) -> Result<Pages> {
        Ok(Pages {
            r#type: dict.need_type(PdfName::Pages, PdfError::InvalidPages)?,
            parent: dict.want_reference(PdfName::Parent),
            kids: dict.need_array(PdfName::Kids, PdfError::InvalidPages)?,
            count: dict.need_u32(PdfName::Count, PdfError::InvalidPages)?,
        })
    }
}

#[derive(Debug)]
pub struct Page {
    pub r#type: PdfName, // must be /Page
    pub parent: Reference,
    pub last_modified: Option<PdfString>,
    pub resources: Option<Dictionary>,
    pub media_box: Option<Array>,
    pub crop_box: Option<Array>,
    pub bleed_box: Option<Array>,
    pub trim_box: Option<Array>,
    pub art_box: Option<Array>,
    pub box_color_info: Option<Dictionary>,
    pub contents: Option<PdfObject>, // stream or array
    pub rotate: Option<i32>,
    pub group: Option<Dictionary>,
    pub thumb: Option<PdfObject>, // stream
    pub dur: Option<PdfNumber>,
    pub trans: Option<Dictionary>,
    pub annots: Option<Array>,
    pub aa: Option<Dictionary>,
    pub metadata: Option<PdfObject>, // stream
    pub piece_info: Option<Dictionary>,
    pub struct_parents: Option<i32>,
    pub id: Option<PdfString>,
    pub pz: Option<PdfNumber>,
    pub separation_info: Option<Dictionary>,
    pub tabs: Option<PdfName>,
    pub template_instantiated: Option<PdfString>,
    pub pres_steps: Option<Dictionary>,
    pub user_unit: Option<PdfNumber>,
    pub vp: Option<Dictionary>,
}

impl Page {
    pub fn new(mut dict: Dictionary) -> Result<Page> {
        Ok(Page {
            r#type: dict.need_type(PdfName::Page, PdfError::InvalidPage)?,
            parent: dict.need_reference(PdfName::Parent, PdfError::InvalidPage)?,
            last_modified: dict.want_string(PdfName::LastModified),
            resources: dict.want_dictionary(PdfName::Resources),
            media_box: dict.want_array(PdfName::MediaBox),
            crop_box: dict.want_array(PdfName::CropBox),
            bleed_box: dict.want_array(PdfName::BleedBox),
            trim_box: dict.want_array(PdfName::TrimBox),
            art_box: dict.want_array(PdfName::ArtBox),
            box_color_info: dict.want_dictionary(PdfName::BoxColorInfo),
            contents: None,
            rotate: dict.want_i32(PdfName::Rotate),
            group: dict.want_dictionary(PdfName::Group),
            thumb: None,
            dur: dict.want_number(PdfName::Dur),
            trans: dict.want_dictionary(PdfName::Trans),
            annots: dict.want_array(PdfName::Annots),
            aa: dict.want_dictionary(PdfName::AA),
            metadata: None,
            piece_info: dict.want_dictionary(PdfName::PieceInfo),
            struct_parents: dict.want_i32(PdfName::StructParents),
            id: dict.want_string(PdfName::ID),
            pz: dict.want_number(PdfName::PZ),
            separation_info: dict.want_dictionary(PdfName::SeparationInfo),
            tabs: dict.want_name(PdfName::Tabs),
            template_instantiated: dict.want_symbol(PdfName::TemplateInstantiated),
            pres_steps: dict.want_dictionary(PdfName::PresSteps),
            user_unit: dict.want_number(PdfName::UserUnit),
            vp: dict.want_dictionary(PdfName::VP),
        })
    }
}
