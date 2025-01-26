# Crusty - Secure Media Storage

A secure media storage application built with Rust running Tokio asynchronous (backend) and React/TypeScript (frontend). Features end-to-end encryption, secure user authentication, and a modern UI. I consider this more of a rust/react template CRUD application that can be adjusted with features as needed. It was built 100% on Windows, paths and variables will need to be adjusted for UNIX.

## Prerequisites

- Node.js (v18 or later)
- Rust (latest stable)
- MySQL (v8.0 or later)
- MySQL Command Line Tools (mysql.exe should be in your PATH)

## Project Structure

```
crusty/
├── backend/           # Rust backend server
│   ├── src/          # Source code
│   ├── Cargo.toml    # Rust dependencies
│   ├── .env.example  # Example environment variables
│   ├── run.ps1.example    # Example server startup script
│   └── init_db.ps1.example # Example database init script
├── frontend/         # React/TypeScript frontend
│   ├── src/         # Source code
│   └── package.json # Node.js dependencies
└── database/        # Database migrations
    └── migrations/  # SQL migration files
```

## Setup Instructions

### 1. Environment Setup

1. Copy example files and update with your credentials:
```powershell
cd backend
copy .env.example .env
copy run.ps1.example run.ps1
copy init_db.ps1.example init_db.ps1
```

2. Edit `.env` with your database credentials:
```
RUST_LOG=debug
DATABASE_URL=mysql://[USERNAME]:[PASSWORD]@localhost:3306/crusty
JWT_SECRET=[YOUR_SECRET_KEY]
STORAGE_PATH=./storage
```

3. Update `run.ps1` and `init_db.ps1` with your MySQL credentials

### 2. Database Setup

Make sure MySQL is running and accessible with your credentials.

Add MySQL to your system PATH:
1. Find your MySQL installation directory (typically `C:\Program Files\MySQL\MySQL Server 8.0\bin`)
2. Add this path to your system's PATH environment variable
3. Restart your terminal/PowerShell

Initialize the database:
```powershell
cd backend
./init_db.ps1
```

### 3. Backend Setup

```powershell
cd backend
cargo build
./run.ps1
```

The backend server will start on http://localhost:5000

### 4. Frontend Setup

```powershell
cd frontend
npm install
npm run dev
```

The frontend development server will start on http://localhost:5173

## Environment Variables

### Backend (.env)
```
RUST_LOG=debug
DATABASE_URL=mysql://root:password@localhost:3306/crusty
JWT_SECRET=sdkjfhsdjkfh3h3
STORAGE_PATH=./storage
```

## Features

- 🔐 Secure user authentication with JWT
- 🔒 End-to-end encrypted file storage
- 📁 File upload and management
- 🎨 Modern, responsive UI
- 🚀 High-performance Rust backend
- 🔍 File search and filtering

## API Endpoints

### Authentication
- POST `/auth/register` - Register a new user
- POST `/auth/login` - Login and receive JWT token

### Media
- POST `/media` - Upload a file
- GET `/media` - List all files
- GET `/media/:id` - Download a file
- DELETE `/media/:id` - Delete a file

## Security Features

- Password hashing using bcrypt
- JWT-based authentication
- File encryption using AES-256-GCM
- CORS protection
- SQL injection prevention
- XSS protection headers

## Development

To run the application in development mode:

1. Start the backend server:
```powershell
cd backend
./run.ps1
```

2. In a new terminal, start the frontend development server:
```powershell
cd frontend
npm run dev
```

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Verify MySQL is running
   - Check credentials in `.env`
   - Ensure database exists

2. **MySQL Command Not Found**
   - Add MySQL to system PATH
   - Verify MySQL installation

3. **Registration/Login Failed**
   - Check backend logs for details
   - Verify database tables exist
   - Ensure correct API URL in frontend

## License

MIT License - Feel free to use this project for personal or commercial purposes.
