pub mod dominos {
    pub const VERTEX_SHADER: &str = r#"#version 330 core
        layout (location = 0) in vec3 pos_model_space;
        layout (location = 1) in vec3 aNormal;
        layout (location = 2) in vec2 aOffset;

        uniform mat4 view_mat;

        out vec3 Normal;
        out vec3 FragPos;
        out vec3 vLightPos;

        void main()
        {
            vec3 pos_world_space = vec3(pos_model_space.x + aOffset.x, pos_model_space.y, pos_model_space.z + aOffset.y);
            Normal = aNormal;
            FragPos = pos_world_space;
            gl_Position = view_mat * vec4(pos_world_space, 1.0);
        }
        "#;
    pub const FRAGMENT_SHADER: &str = 
        r#"#version 330 core
        out vec4 FragColor;

        in vec3 Normal;
        in vec3 FragPos;

        uniform vec3 lightPos;

        void main()
        {
            vec3 lightColor = vec3(1.0, 1.0, 1.0);
            vec3 objectColor = vec3(1.0, 0.0, 1.0);
            
            float ambientStrength = 0.1;
            vec3 ambient = ambientStrength * lightColor;


            vec3 norm = normalize(Normal);
            vec3 lightDir = normalize(lightPos - FragPos);
            float diff = max(dot(norm, lightDir), 0.0);
            vec3 diffuse = diff * lightColor;
            vec3 result = (diffuse + ambient) * objectColor;
            FragColor = vec4(result, 1.0);
        }
        "#;
}
