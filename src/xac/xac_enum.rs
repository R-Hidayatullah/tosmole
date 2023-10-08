#![allow(dead_code)]
pub(crate) enum XacChunkType {
    XacMeshId = 1,
    XacSkinningId = 2,
    XacMaterialDefinitionId = 3,
    XacShaderMaterialId = 5,
    XacMetadataId = 7,
    XacNodeHierarchyId = 11,
    XacMorphTargetId = 12,
    XacMaterialTotalId = 13,
}

pub(crate) enum XacVerticesAttributeType {
    XacPositionId = 0,
    XacNormalId = 1,
    XacTangentId = 2,
    XacUVCoordId = 3,
    XacColor32Id = 4,
    XacInfluenceRangeId = 5,
    XacColor128Id = 6,
}
