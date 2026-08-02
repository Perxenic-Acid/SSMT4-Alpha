use crate::common::frame_analysis::frameanalysis_data::FrameAnalysisData;
use crate::common::frame_analysis::frameanalysis_log::FrameAnalysisSingleLog;

pub struct FrameAnalysis {
    pub folder_path: String,
    pub data: FrameAnalysisData,
    pub log: FrameAnalysisSingleLog,
}

impl FrameAnalysis {
    pub fn new(frame_analysis_folder: &str) -> Result<Self, String> {
        let fa_log = FrameAnalysisSingleLog::new(frame_analysis_folder)?;
        let fa_data = FrameAnalysisData::new(frame_analysis_folder)?;
        Ok(Self {
            folder_path: frame_analysis_folder.to_string(),
            data: fa_data,
            log: fa_log,
        })
    }
}
