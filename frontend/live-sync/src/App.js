import React, {useState, useEffect, useMemo} from 'react';
import "bulma/css/bulma.css";

let socketConn = null;

function App() {
  const [courses, setCourses] = useState([]);
  const [username, setUsername] = useState("");
  const [deletec, setDeletec] = useState("");
  const [addc, setAddc] = useState("");

  const handleChange = useMemo(() => {
    return (setHandler) => (e) => {
      setHandler(e.target.value);
    }
  }, []);
  

  useEffect(() => {
    socketConn = new WebSocket('ws://localhost:8000/ws/');
    console.log(socketConn);
    socketConn.addEventListener('message', (e) => {
      // assume well formed course info
      // {courses: []}
      console.log(e.data);
      let msg = JSON.parse(e.data);
      msg.sort();
      setCourses(msg);
    })
  }, [setCourses]);
  
  
  return (
    <>
      <nav className="navbar is-link" role="navigation">
        <div className="navbar-menu">
          <div className="navbar-start">
            <p className="navbar-item">
              Demo
            </p>
          </div>
        </div>
        
      </nav>

      <section className="section">
        <div className="container">
          <h1 className="title">
            Sync Demo
          </h1>
          <div className="tile is-ancestor">
            <div className="tile is-child box">
              <p>Courses</p>
              {courses.map(c => {
                return <p key={c}>{c}</p>
              })}
            </div>
            <div className="tile is-child box">
              <p>Options</p>
              <div style={{margin: "1rem"}}>
                <input className="input is-small" type="text" style={{width: "15rem"}}
                  placeholder="username" value={username} onChange={handleChange(setUsername)}/>
                <button className="button is-small" onClick={() => {
                  socketConn.send(`/join ${username}`);
                }}> Submit</button>
                <input className="input is-small" type="text" style={{width: "15rem"}}
                  placeholder="delete" value={deletec} onChange={handleChange(setDeletec)}/>
                <button className="button is-small" onClick={() => {
                  socketConn.send(`/rem ${deletec}`);
                }}> Submit</button>
                <input className="input is-small" type="text" style={{width: "15rem"}}
                  placeholder="add" value={addc} onChange={handleChange(setAddc)}/>
                <button className="button is-small" onClick={() => {
                  socketConn.send(`/add ${addc}`);
                }}> Submit</button>

              </div>
            </div>
          </div>

        </div>
      </section>


    </>
  );
}

export default App;
