using System;
using System.Collections.ObjectModel;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic
{
    public class AdminManager : NotifyBase
    {
        public ObservableCollection<Log> Logs { get; } = new ObservableCollection<Log>();

        DateTimeOffset date = DateTimeOffset.Now;
        public DateTimeOffset Date
        {
            get => date;
            set { date = value; GetLogs(); OnPropertyChanged(); }
        }

        public AdminManager()
        {
            GetLogs();
            Log.LogsSaved += Log_LogsSaved;
        }

        private void Log_LogsSaved(object sender, EventArgs e)
        {
            GetLogs();
        }

        async void GetLogs()
        {
            Logs.Clear();
            foreach (var log in await Log.GetLogs(Date))
            {
                Logs.Add(log);
            }
        }
    }
}
