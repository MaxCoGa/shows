# shows
self hosted open web services


# pCloud API

This is a simple file storage and authentication API built with Rust and Axum.

## Running the Application

1.  Start the application:
    ```bash
    cargo run
    ```
2.  The application will start on a local port (e.g., `http://127.0.0.1:3000`). Check the terminal output for the exact address.

## API Endpoints

### Authentication

#### Register a New User

-   **Endpoint**: `POST /auth/register`
-   **Description**: Creates a new user.
-   **Request Body**: A JSON object with `username` and `password`.

    ```json
    {
      "username": "newuser",
      "password": "newpassword"
    }
    ```

-   **Success Response**: A JSON object with a success message.

    ```json
    {
      "message": "User created successfully"
    }
    ```

-   **`curl` command**:
    ```bash
    curl -X POST -H "Content-Type: application/json" -d '{"username":"newuser","password":"newpassword"}' http://127.0.0.1:3000/auth/register
    ```

#### User Login

-   **Endpoint**: `POST /auth/login`
-   **Description**: Authenticates a user and returns a JWT.
-   **Request Body**: A JSON object with `username` and `password`.

    ```json
    {
      "username": "testuser",
      "password": "testpassword"
    }
    ```

-   **Success Response**: A JSON object with a JWT token.

    ```json
    {
      "token": "your.jwt.token"
    }
    ```

-   **`curl` command**:
    ```bash
    curl -X POST -H "Content-Type: application/json" -d '{"username":"testuser","password":"testpassword"}' http://127.0.0.1:3000/auth/login
    ```

#### Accessing Protected Routes

Once you have a JWT token, you can access protected routes by including it in the `Authorization` header of your requests. The format is `Bearer <your.jwt.token>`.

-   **Endpoint**: `GET /auth/protected`
-   **Description**: An example of a protected route.
-   **`curl` command**:

    ```bash
    curl -H "Authorization: Bearer <your.jwt.token>" http://127.0.0.1:3000/auth/protected
    ```

#### Delete a User

-   **Endpoint**: `DELETE /auth/user`
-   **Description**: Deletes the currently authenticated user.
-   **Headers**: `Authorization: Bearer <your.jwt.token>`
-   **Success Response**: A JSON object with a success message.

    ```json
    {
      "message": "User deleted successfully"
    }
    ```

-   **`curl` command**:
    ```bash
    curl -X DELETE -H "Authorization: Bearer <your.jwt.token>" http://127.0.0.1:3000/auth/user
    ```

### File Management

**Note:** All endpoints in this section are protected and require a valid JWT token.

#### List Files

-   **Endpoint**: `GET /api/files`
-   **Description**: Retrieves a list of all files stored in the `pcloud_files` directory.
-   **Response**: A JSON array of file objects.

    ```json
    [
      {
        "name": "file1.txt"
      },
      {
        "name": "image.png"
      }
    ]
    ```

-   **`curl` command**:
    ```bash
    curl -H "Authorization: Bearer <your.jwt.token>" http://127.0.0.1:3000/api/files
    ```

#### Upload a File

-   **Endpoint**: `POST /api/files`
-   **Description**: Uploads a single file. The request must be a `multipart/form-data` request with a form field named `file`.
-   **Success Response**: A string confirming the successful upload.
-   **`curl` command**:
    ```bash
    curl -X POST -H "Authorization: Bearer <your.jwt.token>" -F "file=@/path/to/your/file.txt" http://127.0.0.1:3000/api/files
    ```

#### Download a File

-   **Endpoint**: `GET /api/files/:filename`
-   **Description**: Downloads a specific file from the `pcloud_files` directory.
-   **Success Response**: The file content as an octet stream.
-   **`curl` command**:
    ```bash
    curl -H "Authorization: Bearer <your.jwt.token>" http://127.0.0.1:3000/api/files/your_file.txt --output your_file.txt
    ```

#### Delete a File

-   **Endpoint**: `DELETE /api/files/:filename`
-   **Description**: Deletes a specific file from the `pcloud_files` directory.
-   **Success Response**: A string confirming the successful deletion.
-   **`curl` command**:
    ```bash
    curl -X DELETE -H "Authorization: Bearer <your.jwt.token>" http://127.0.0.1:3000/api/files/your_file.txt
    ```
