use dictionary::Access;
use errors::*;
use pdf_types::*;

type Result<T> = ::std::result::Result<T, PdfError>;

#[derive(Debug)]
pub struct Pages {
    pub r#type: PdfName, // must be /Pages
    pub parent: Option<Reference>,
    pub kids: Array,
    pub count: u32,
    // inheritable
    pub resources: Option<Dictionary>,
    pub media_box: Option<Array>,
    pub crop_box: Option<Array>,
    pub rotate: Option<i32>,
}

impl Pages {
    pub fn new_root(mut dict: Dictionary) -> Result<Pages> {
        Ok(Pages {
            r#type: dict.need_type(PdfName::Pages, PdfError::InvalidPages)?,
            parent: dict.want_reference(PdfName::Parent),
            kids: dict.need_array(PdfName::Kids, PdfError::InvalidPages)?,
            count: dict.need_u32(PdfName::Count, PdfError::InvalidPages)?,
            // inheritable
            resources: dict.want_dictionary(PdfName::Resources),
            media_box: dict.want_array(PdfName::MediaBox),
            crop_box: dict.want_array(PdfName::CropBox),
            rotate: dict.want_i32(PdfName::Rotate),
        })
    }

    pub fn new(mut dict: Dictionary, parent: &Pages) -> Result<Pages> {
        let resources = match (dict.want_dictionary(PdfName::Resources), &parent.resources) {
            (None, r) => r.clone(),
            (r, _) => r,
        };
        let media_box = match (dict.want_array(PdfName::MediaBox), &parent.media_box) {
            (None, mb) => mb.clone(),
            (mb, _) => mb,
        };
        let crop_box = match (dict.want_array(PdfName::CropBox), &parent.crop_box) {
            (None, cb) => cb.clone(),
            (cb, _) => cb,
        };
        let rotate = match (dict.want_i32(PdfName::Rotate), parent.rotate) {
            (None, r) => r,
            (r, _) => r,
        };
        Ok(Pages {
            r#type: dict.need_type(PdfName::Pages, PdfError::InvalidPages)?,
            parent: dict.want_reference(PdfName::Parent),
            kids: dict.need_array(PdfName::Kids, PdfError::InvalidPages)?,
            count: dict.need_u32(PdfName::Count, PdfError::InvalidPages)?,
            // inheritable
            resources: resources,
            media_box: media_box,
            crop_box: crop_box,
            rotate: rotate,
        })
    }
}

#[derive(Debug)]
pub struct Page {
    pub r#type: PdfName, // must be /Page
    pub parent: Reference,
    pub last_modified: Option<PdfString>,
    pub resources: Dictionary,
    pub media_box: Array,
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
    pub fn new(mut dict: Dictionary, parent: &Pages) -> Result<Page> {
        let resources: Result<Dictionary> =
            match (dict.want_dictionary(PdfName::Resources), &parent.resources) {
                (Some(r), _) => Ok(r),
                (None, Some(r)) => Ok(r.clone()),
                (None, None) => Err(PdfError::InvalidPage),
            };
        let media_box: Result<Array> = match (dict.want_array(PdfName::MediaBox), &parent.media_box)
        {
            (Some(mb), _) => Ok(mb),
            (None, Some(mb)) => Ok(mb.clone()),
            (None, None) => Err(PdfError::InvalidPage),
        };
        let crop_box = match (dict.want_array(PdfName::CropBox), &parent.crop_box) {
            (None, cb) => cb.clone(),
            (cb, _) => cb,
        };
        let rotate = match (dict.want_i32(PdfName::Rotate), parent.rotate) {
            (None, r) => r,
            (r, _) => r,
        };
        Ok(Page {
            r#type: dict.need_type(PdfName::Page, PdfError::InvalidPage)?,
            parent: dict.need_reference(PdfName::Parent, PdfError::InvalidPage)?,
            last_modified: dict.want_string(PdfName::LastModified),
            resources: resources?,
            media_box: media_box?,
            crop_box: crop_box,
            bleed_box: dict.want_array(PdfName::BleedBox),
            trim_box: dict.want_array(PdfName::TrimBox),
            art_box: dict.want_array(PdfName::ArtBox),
            box_color_info: dict.want_dictionary(PdfName::BoxColorInfo),
            contents: None,
            rotate: rotate,
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
