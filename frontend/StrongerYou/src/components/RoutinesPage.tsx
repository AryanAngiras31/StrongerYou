import React, { useEffect, useState } from 'react';
import { IonContent, IonPage, IonHeader, IonToolbar, IonTitle, IonButton, IonIcon, IonRow, IonCol } from '@ionic/react';
import { ellipsisVertical } from 'ionicons/icons';
import './RoutinesPage.css';

interface Routine {
  routine_id: number;
  name: string;
  timestamp: string;
  last_performed: string | null;
}

const RoutinesPage: React.FC = () => {
  const [routines, setRoutines] = useState<Routine[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchRoutines = async () => {
      try {
        const response = await fetch('http://127.0.0.1:8080/routines?sort=createdAt&include=lastPerformed');
        
        if (!response.ok) {
          throw new Error('Failed to fetch routines');
        }
        
        const data = await response.json();
        setRoutines(data);
      } catch (err) {
        setError('Failed to load routines. Please try again later.');
        console.error('Error fetching routines:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchRoutines();
  }, []);

  const formatDate = (dateString: string | null) => {
    if (!dateString) return 'Never';
    
    const date = new Date(dateString);
    // Format as MM/DD/YY
    return `${(date.getMonth() + 1).toString().padStart(2, '0')}/${date.getDate().toString().padStart(2, '0')}/${date.getFullYear().toString().slice(-2)}`;
  };

  return (
    <IonPage>
      <IonHeader className="ion-no-border">
        <IonToolbar className="app-header">
          <IonTitle className="app-title">StrongerYou</IonTitle>
        </IonToolbar>
      </IonHeader>
      
      <IonContent fullscreen className="ion-padding">
        <h1 className="routines-title">Your Routines</h1>
        
        {loading && <p className="loading-text">Loading your routines...</p>}
        {error && <p className="error-text">{error}</p>}
        
        {!loading && !error && (
          <div className="routines-container">
            {routines.map((routine) => (
              <div key={routine.routine_id} className="routine-card">
                <div className="routine-info">
                  <span className="routine-name">{routine.name}</span>
                  <span className="last-performed">Last performed {formatDate(routine.last_performed)}</span>
                </div>
                <div className="routine-actions">
                  <IonButton expand="block" className="start-button">
                    Start
                  </IonButton>
                  <div className="more-options">
                    <IonIcon icon={ellipsisVertical} />
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
        
        <div className="navigation-bar">
          <div className="nav-button">
            <div className="nav-icon home-icon"></div>
          </div>
          <div className="nav-button">
            <div className="nav-icon timer-icon">0:0</div>
          </div>
          <div className="nav-button">
            <div className="nav-icon profile-icon"></div>
          </div>
        </div>
      </IonContent>
    </IonPage>
  );
};

export default RoutinesPage;