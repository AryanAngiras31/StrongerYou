import React, { useEffect, useState } from 'react';
import { IonContent, IonPage, IonHeader, IonToolbar, IonTitle, IonButton, IonIcon } from '@ionic/react';
import { ellipsisVertical, add } from 'ionicons/icons';
import { list_routines } from '../services/api';
import type { RoutineInfo } from '../services/types';
import './Routines.css';

const RoutinesPage: React.FC = () => {
  const [routines, setRoutines] = useState<RoutineInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    const fetchRoutines = async () => {
      try {
        const data = await list_routines(true);
        setRoutines(data);
      } catch (err) {
        console.error('Full error details:', err);
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
        
        {!loading && (
          <div className="routines-container">
            {routines.length === 0 ? (
              <div className="no-routines-container">
                <p className="no-routines">No routines found. Create a routine by clicking the '+' icon.</p>
                <IonButton className="add-routine-button">
                  <IonIcon icon={add} />
                </IonButton>
              </div>
            ) : (
              routines.map((routine) => (
                <div key={routine.routine_id} className="routine-card">
                  <div className="routine-header">
                    <div className="routine-title">{routine.name}</div>
                    <div className="more-options">
                      <IonIcon icon={ellipsisVertical} />
                    </div>
                  </div>
                  <div className="routine-last-performed">
                    last performed {formatDate(routine.last_performed)}
                  </div>
                  <IonButton expand="block" className="start-button">
                    <div className="start-text">
                      Start
                    </div>
                  </IonButton>
                </div>
              ))
            )}
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