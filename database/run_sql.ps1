$sql = Get-Content -Path "verify_db.sql" -Raw
& 'C:\Program Files\MySQL\MySQL Server 8.0\bin\mysql.exe' -u root -pTesla86!! --execute="$sql" 