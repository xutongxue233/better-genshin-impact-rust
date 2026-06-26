use super::FileMethodBinding;

pub(in crate::runtime_bindings) const FILE_METHODS: &[FileMethodBinding] = &[
    FileMethodBinding {
        property_name: "readPathSync",
        host_method: "readPathSync",
        length: 1,
    },
    FileMethodBinding {
        property_name: "ReadPathSync",
        host_method: "ReadPathSync",
        length: 1,
    },
    FileMethodBinding {
        property_name: "createDirectory",
        host_method: "createDirectory",
        length: 1,
    },
    FileMethodBinding {
        property_name: "CreateDirectory",
        host_method: "CreateDirectory",
        length: 1,
    },
    FileMethodBinding {
        property_name: "isFolder",
        host_method: "isFolder",
        length: 1,
    },
    FileMethodBinding {
        property_name: "IsFolder",
        host_method: "IsFolder",
        length: 1,
    },
    FileMethodBinding {
        property_name: "isFile",
        host_method: "isFile",
        length: 1,
    },
    FileMethodBinding {
        property_name: "IsFile",
        host_method: "IsFile",
        length: 1,
    },
    FileMethodBinding {
        property_name: "isExists",
        host_method: "isExists",
        length: 1,
    },
    FileMethodBinding {
        property_name: "IsExists",
        host_method: "IsExists",
        length: 1,
    },
    FileMethodBinding {
        property_name: "readTextSync",
        host_method: "readTextSync",
        length: 1,
    },
    FileMethodBinding {
        property_name: "ReadTextSync",
        host_method: "ReadTextSync",
        length: 1,
    },
    FileMethodBinding {
        property_name: "readText",
        host_method: "readText",
        length: 1,
    },
    FileMethodBinding {
        property_name: "ReadText",
        host_method: "ReadText",
        length: 1,
    },
    FileMethodBinding {
        property_name: "writeTextSync",
        host_method: "writeTextSync",
        length: 2,
    },
    FileMethodBinding {
        property_name: "WriteTextSync",
        host_method: "WriteTextSync",
        length: 2,
    },
    FileMethodBinding {
        property_name: "writeText",
        host_method: "writeText",
        length: 2,
    },
    FileMethodBinding {
        property_name: "WriteText",
        host_method: "WriteText",
        length: 2,
    },
    FileMethodBinding {
        property_name: "readImageMatSync",
        host_method: "readImageMatSync",
        length: 1,
    },
    FileMethodBinding {
        property_name: "ReadImageMatSync",
        host_method: "ReadImageMatSync",
        length: 1,
    },
    FileMethodBinding {
        property_name: "readImageMatWithResizeSync",
        host_method: "readImageMatWithResizeSync",
        length: 3,
    },
    FileMethodBinding {
        property_name: "ReadImageMatWithResizeSync",
        host_method: "ReadImageMatWithResizeSync",
        length: 3,
    },
    FileMethodBinding {
        property_name: "writeImageSync",
        host_method: "writeImageSync",
        length: 2,
    },
    FileMethodBinding {
        property_name: "WriteImageSync",
        host_method: "WriteImageSync",
        length: 2,
    },
    FileMethodBinding {
        property_name: "renamePathSync",
        host_method: "renamePathSync",
        length: 2,
    },
    FileMethodBinding {
        property_name: "RenamePathSync",
        host_method: "RenamePathSync",
        length: 2,
    },
];
