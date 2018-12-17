using System;
using System.Collections.ObjectModel;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic
{
    public class AdminManager
    {
        public AdminManager()
        {
            GetTodaysLogs();
            Log.LogsSaved += Log_LogsSaved;
        }

        private void Log_LogsSaved(object sender, EventArgs e)
        {
            GetTodaysLogs();
        }

        async void GetTodaysLogs()
        {
            TodaysLogs.Clear();
            foreach (var log in await Log.GetLogs(DateTime.Today))
            {
                TodaysLogs.Add(log);
            }
        }

        public ObservableCollection<Log> TodaysLogs { get; } = new ObservableCollection<Log>();
    }
}
