use rusqlite::{Connection, Result, params};
use crate::app::domain::employee::{Employee, NewEmployee};

pub struct EmployeeRepo<'c> {
    pub conn: &'c Connection,
}

impl<'c> EmployeeRepo<'c> {
    pub fn list_all(&self) -> Result<Vec<Employee>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, nik, name, department, position, base_salary, is_active
             FROM employees",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Employee {
                id: row.get(0)?,
                nik: row.get(1)?,
                name: row.get(2)?,
                department: row.get(3)?,
                position: row.get(4)?,
                base_salary: row.get(5)?,
                is_active: row.get::<_, i64>(6)? != 0,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn insert(&self, e: &NewEmployee) -> Result<()> {
        self.conn.execute(
            "INSERT INTO employees(nik, name, department, position, base_salary, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![
                e.nik,
                e.name,
                e.department,
                e.position,
                e.base_salary,
            ],
        )?;
        Ok(())
    }
}
