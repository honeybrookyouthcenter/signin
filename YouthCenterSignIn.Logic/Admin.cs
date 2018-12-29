using System;
using System.Collections.ObjectModel;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic
{
    public class Admin : NotifyBase
    {
        DataProvider DataProvider => DataProvider.Current;

        public ObservableCollection<Log> Logs { get; } = new ObservableCollection<Log>();

        DateTimeOffset date = DateTimeOffset.Now;
        public DateTimeOffset Date
        {
            get => date;
            set { date = value; GetLogs(); OnPropertyChanged(); }
        }

        public Admin()
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

        #region Authentication

        public bool Authenticate(string pin)
        {
            return pin == Pin;
        }

        public bool ChangeAdminPin(string currentPin, string newPin, string newPinConfirm)
        {
            void ShowMessage(string message) => DataProvider.ShowMessage(message);

            if (currentPin != Pin)
            {
                ShowMessage("Incorrect PIN.");
                return false;
            }

            if (newPin == Pin)
            {
                ShowMessage("The new PIN is the same as the current PIN.");
                return false;
            }

            if (newPin.Length != 6)
            {
                ShowMessage("The PIN must be 6 characters long.");
                return false;
            }

            if (newPin != newPinConfirm)
            {
                ShowMessage("The confirmation PIN is not the same.");
                return false;
            }

            Pin = newPin;
            return true;
        }

        public const string DefaultPin = "123123";
        public string Pin
        {
            get
            {
                //Waiting is ok because app settings don't need async
                var savedPin = DataProvider.GetSetting<string>(nameof(Pin), StorageType.AppSetting).Result;
                return savedPin ?? DefaultPin;
            }

            private set
            {
                var saveTask = DataProvider.SetSetting(nameof(Pin), value, StorageType.AppSetting);
            }
        }

        #endregion
    }
}
