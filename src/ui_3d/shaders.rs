pub mod dominos {
    pub const VERTEX_SHADER: &str = r#"#version 330 core
        layout (location = 0) in vec3 pos_model_space;
        layout (location = 1) in vec3 aNormal;
        layout (location = 3) in mat4 model_mat;
        layout (location = 7) in mat4 rot_mat;
        layout (location = 11) in float id;

        uniform mat4 view_mat;
        uniform mat4 perspective_mat;

        out vec3 Normal;
        out vec3 FragPos;
        flat out float out_id;

        void main()
        {
            Normal = vec3(rot_mat * vec4(aNormal, 1.0));
            vec3 pos_world_space = pos_model_space;
            FragPos = vec3(model_mat * vec4(pos_world_space, 1.0));
            mat4 mvp = perspective_mat * view_mat * model_mat;
            gl_Position = mvp * vec4(pos_world_space, 1.0);

            out_id = id;
        }
        "#;
    pub const FRAGMENT_SHADER: &str =
        r#"#version 330 core
        out vec4 FragColor;

        in vec3 Normal;
        in vec3 FragPos;
        flat in float out_id;

        uniform vec3 lightPos;
        uniform vec3 camPos;
        uniform float selected_id;

        void main()
        {
            vec3 lightColor = vec3(1.0, 1.0, 1.0);
            vec3 objectColor = vec3(1.0, 0.0, 0.0); 
            if (selected_id == out_id) {
                objectColor = vec3(0.0, 0.0, 1.0);
            }

            float ambientStrength = 0.1;
            vec3 ambient = ambientStrength * lightColor;

            vec3 norm = normalize(Normal);
            vec3 lightDir = normalize(lightPos - FragPos);
            float diff = max(dot(norm, lightDir), 0.0);
            vec3 diffuse = diff * lightColor;

            float specularStrength = 0.5;
            vec3 viewDir = normalize(camPos - FragPos);
            vec3 reflectDir = reflect(-lightDir, norm);
            float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
            vec3 specular = specularStrength * spec * lightColor;

            vec3 result = (diffuse + ambient + specular) * objectColor;
            FragColor = vec4(result, 1.0);
        }
        "#;

    pub fn get_vertices(dimensions: cgmath::Vector3<f32>) -> [f32; 216] {
        [
            -dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  0.0f32,  0.0f32, -1.0f32,
             dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  0.0f32,  0.0f32, -1.0f32, 
             dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  0.0f32,  0.0f32, -1.0f32, 
             dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  0.0f32,  0.0f32, -1.0f32, 
            -dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  0.0f32,  0.0f32, -1.0f32, 
            -dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  0.0f32,  0.0f32, -1.0f32, 

            -dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  0.0f32,  0.0f32, 1.0f32,
             dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  0.0f32,  0.0f32, 1.0f32,
             dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  0.0f32,  0.0f32, 1.0f32,
             dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  0.0f32,  0.0f32, 1.0f32,
            -dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  0.0f32,  0.0f32, 1.0f32,
            -dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  0.0f32,  0.0f32, 1.0f32,

            -dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5, -1.0f32,  0.0f32,  0.0f32,
            -dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5, -1.0f32,  0.0f32,  0.0f32,
            -dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5, -1.0f32,  0.0f32,  0.0f32,
            -dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5, -1.0f32,  0.0f32,  0.0f32,
            -dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5, -1.0f32,  0.0f32,  0.0f32,
            -dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5, -1.0f32,  0.0f32,  0.0f32,

             dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  1.0f32,  0.0f32,  0.0f32,
             dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  1.0f32,  0.0f32,  0.0f32,
             dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  1.0f32,  0.0f32,  0.0f32,
             dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  1.0f32,  0.0f32,  0.0f32,
             dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  1.0f32,  0.0f32,  0.0f32,
             dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  1.0f32,  0.0f32,  0.0f32,

            -dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  0.0f32, -1.0f32,  0.0f32,
             dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  0.0f32, -1.0f32,  0.0f32,
             dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  0.0f32, -1.0f32,  0.0f32,
             dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  0.0f32, -1.0f32,  0.0f32,
            -dimensions.x * 0.5, 0.0f32,        dimensions.z * 0.5,  0.0f32, -1.0f32,  0.0f32,
            -dimensions.x * 0.5, 0.0f32,       -dimensions.z * 0.5,  0.0f32, -1.0f32,  0.0f32,

            -dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  0.0f32,  1.0f32,  0.0f32,
             dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  0.0f32,  1.0f32,  0.0f32,
             dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  0.0f32,  1.0f32,  0.0f32,
             dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  0.0f32,  1.0f32,  0.0f32,
            -dimensions.x * 0.5, dimensions.y,  dimensions.z * 0.5,  0.0f32,  1.0f32,  0.0f32,
            -dimensions.x * 0.5, dimensions.y, -dimensions.z * 0.5,  0.0f32,  1.0f32,  0.0f32
        ]
    }
}

pub mod light_source {
    pub const VERTEX_SHADER: &str = r#"#version 330 core
        layout (location = 0) in vec3 pos_model_space;

        uniform mat4 view_mat;
        uniform mat4 perspective_mat;
        uniform vec3 lightPos;

        void main()
        {
            vec3 pos_world_space = pos_model_space + lightPos;
            mat4 mvp = perspective_mat * view_mat;
            gl_Position = mvp * vec4(pos_world_space, 1.0);
        }
        "#;
    pub const FRAGMENT_SHADER: &str = 
        r#"#version 330 core
        out vec4 FragColor;

        void main()
        {
            FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
        "#;

    pub const VERTICES: [f32; 216] = [
        -0.1f32, -0.1f32, -0.1f32,  0.0f32,  0.0f32, -1.0f32,
         0.1f32, -0.1f32, -0.1f32,  0.0f32,  0.0f32, -1.0f32, 
         0.1f32,  0.1f32, -0.1f32,  0.0f32,  0.0f32, -1.0f32, 
         0.1f32,  0.1f32, -0.1f32,  0.0f32,  0.0f32, -1.0f32, 
        -0.1f32,  0.1f32, -0.1f32,  0.0f32,  0.0f32, -1.0f32, 
        -0.1f32, -0.1f32, -0.1f32,  0.0f32,  0.0f32, -1.0f32, 

        -0.1f32, -0.1f32,  0.1f32,  0.0f32,  0.0f32, 1.0f32,
         0.1f32, -0.1f32,  0.1f32,  0.0f32,  0.0f32, 1.0f32,
         0.1f32,  0.1f32,  0.1f32,  0.0f32,  0.0f32, 1.0f32,
         0.1f32,  0.1f32,  0.1f32,  0.0f32,  0.0f32, 1.0f32,
        -0.1f32,  0.1f32,  0.1f32,  0.0f32,  0.0f32, 1.0f32,
        -0.1f32, -0.1f32,  0.1f32,  0.0f32,  0.0f32, 1.0f32,

        -0.1f32,  0.1f32,  0.1f32, -1.0f32,  0.0f32,  0.0f32,
        -0.1f32,  0.1f32, -0.1f32, -1.0f32,  0.0f32,  0.0f32,
        -0.1f32, -0.1f32, -0.1f32, -1.0f32,  0.0f32,  0.0f32,
        -0.1f32, -0.1f32, -0.1f32, -1.0f32,  0.0f32,  0.0f32,
        -0.1f32, -0.1f32,  0.1f32, -1.0f32,  0.0f32,  0.0f32,
        -0.1f32,  0.1f32,  0.1f32, -1.0f32,  0.0f32,  0.0f32,

         0.1f32,  0.1f32,  0.1f32,  1.0f32,  0.0f32,  0.0f32,
         0.1f32,  0.1f32, -0.1f32,  1.0f32,  0.0f32,  0.0f32,
         0.1f32, -0.1f32, -0.1f32,  1.0f32,  0.0f32,  0.0f32,
         0.1f32, -0.1f32, -0.1f32,  1.0f32,  0.0f32,  0.0f32,
         0.1f32, -0.1f32,  0.1f32,  1.0f32,  0.0f32,  0.0f32,
         0.1f32,  0.1f32,  0.1f32,  1.0f32,  0.0f32,  0.0f32,

        -0.1f32, -0.1f32, -0.1f32,  0.0f32, -1.0f32,  0.0f32,
         0.1f32, -0.1f32, -0.1f32,  0.0f32, -1.0f32,  0.0f32,
         0.1f32, -0.1f32,  0.1f32,  0.0f32, -1.0f32,  0.0f32,
         0.1f32, -0.1f32,  0.1f32,  0.0f32, -1.0f32,  0.0f32,
        -0.1f32, -0.1f32,  0.1f32,  0.0f32, -1.0f32,  0.0f32,
        -0.1f32, -0.1f32, -0.1f32,  0.0f32, -1.0f32,  0.0f32,

        -0.1f32,  0.1f32, -0.1f32,  0.0f32,  1.0f32,  0.0f32,
         0.1f32,  0.1f32, -0.1f32,  0.0f32,  1.0f32,  0.0f32,
         0.1f32,  0.1f32,  0.1f32,  0.0f32,  1.0f32,  0.0f32,
         0.1f32,  0.1f32,  0.1f32,  0.0f32,  1.0f32,  0.0f32,
        -0.1f32,  0.1f32,  0.1f32,  0.0f32,  1.0f32,  0.0f32,
        -0.1f32,  0.1f32, -0.1f32,  0.0f32,  1.0f32,  0.0f32
    ];
}

pub mod ground_plane {
    pub const VERTEX_SHADER: &str = r#"#version 330 core
        layout (location = 0) in vec3 pos_model_space;
        layout (location = 1) in vec3 aNormal;

        uniform mat4 view_mat;
        uniform mat4 perspective_mat;

        out vec3 Normal;
        out vec3 FragPos;

        void main()
        {
            vec3 pos_world_space = pos_model_space;
            Normal = aNormal;
            FragPos = pos_world_space;
            mat4 mvp = perspective_mat * view_mat;
            gl_Position = mvp * vec4(pos_world_space, 1.0);
        }
        "#;
    pub const FRAGMENT_SHADER: &str =
        r#"#version 330 core
        out vec4 FragColor;

        in vec3 Normal;
        in vec3 FragPos;

        uniform vec3 lightPos;
        uniform vec3 camPos;

        void main()
        {
            vec3 lightColor = vec3(1.0, 1.0, 1.0);
            vec3 objectColor = vec3(0.0, 1.0, 1.0);
            
            float ambientStrength = 0.1;
            vec3 ambient = ambientStrength * lightColor;

            vec3 norm = normalize(Normal);
            vec3 lightDir = normalize(lightPos - FragPos);
            float diff = max(dot(norm, lightDir), 0.0);
            vec3 diffuse = diff * lightColor;

            float specularStrength = 0.5;
            vec3 viewDir = normalize(camPos - FragPos);
            vec3 reflectDir = reflect(-lightDir, norm);
            float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
            vec3 specular = specularStrength * spec * lightColor;

            vec3 result = (diffuse + ambient + specular) * objectColor;
            FragColor = vec4(result, 1.0);
        }
        "#;

    pub const VERTICES: [f32; 36] = [
        -20f32,  0f32, -20f32,  0.0f32,  1.0f32,  0.0f32,
         20f32,  0f32, -20f32,  0.0f32,  1.0f32,  0.0f32,
         20f32,  0f32,  20f32,  0.0f32,  1.0f32,  0.0f32,
         20f32,  0f32,  20f32,  0.0f32,  1.0f32,  0.0f32,
        -20f32,  0f32,  20f32,  0.0f32,  1.0f32,  0.0f32,
        -20f32,  0f32, -20f32,  0.0f32,  1.0f32,  0.0f32
    ];
}
