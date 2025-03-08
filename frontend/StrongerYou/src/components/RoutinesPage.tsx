import React, { useEffect, useState } from 'react';
import { IonContent, IonPage, IonHeader, IonToolbar, IonTitle, IonButton, IonIcon, IonToast } from '@ionic/react';
import { ellipsisVertical, add } from 'ionicons/icons';
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
  const [showToast, setShowToast] = useState<boolean>(false);
  const [detailedError, setDetailedError] = useState<string>('');

  useEffect(() => {
    const fetchRoutines = async () => {
      try {
        console.log('Fetching routines...');
        const response = await fetch('http://127.0.0.1:8080/routines?sort=createdAt&include=lastPerformed', {
          mode: 'cors',
          headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
          }
        });
        
        console.log('Response status:', response.status);
        console.log('Response headers:', [...response.headers.entries()]);
        
        if (!response.ok) {
          const errorText = await response.text();
          console.error('Error response body:', errorText);
          throw new Error(`Server responded with status: ${response.status}. Details: ${errorText}`);
        }
        
        const data = await response.json();
        console.log('Received data:', data);
        setRoutines(data);
      } catch (err) {
        console.error('Full error details:', err);
        const errorMessage = err instanceof Error ? err.message : 'Unknown error occurred';
        setError('Failed to load routines. Please try again later.');
        setDetailedError(`Technical details: ${errorMessage}`);
        setShowToast(true);
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
        
        {error && (
          <div className="error-container">
            <p className="error-text">{error}</p>
            <p className="error-details">{detailedError}</p>
            <IonButton 
              className="debug-button" 
              size="small" 
              onClick={() => console.log('Open console to see detailed error logs')}
            >
              View Debug Logs
            </IonButton>
          </div>
        )}
        
        {!loading && !error && (
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
                    Start
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
      
      <IonToast
        isOpen={showToast}
        onDidDismiss={() => setShowToast(false)}
        message={detailedError}
        duration={10000}
        position="bottom"
        buttons={[{
          text: 'Dismiss',
          role: 'cancel'
        }]}
      />
    </IonPage>
  );
};

export default RoutinesPage;