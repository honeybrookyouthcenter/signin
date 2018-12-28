using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace YouthCenterSignIn.Logic.Data
{
    public abstract class DataProvider
    {
        public static DataProvider Current { get; set; }

        #region Json File

        public async Task<T> GetSetting<T>(string key, StorageType storageType)
        {
            string json = "";
            switch (storageType)
            {
                case StorageType.AppSetting:
                    json = GetJsonSetting(key);
                    break;
                case StorageType.LocalFile:
                    json = await GetJsonFileContent(key);
                    break;
            }

            if (string.IsNullOrWhiteSpace(json))
                return default(T);
            else
                return JsonConvert.DeserializeObject<T>(json);
        }

        protected abstract string GetJsonSetting(string key);
        protected abstract Task<string> GetJsonFileContent(string file);

        public async Task SetSetting<T>(string key, T value, StorageType storageType)
        {
            string json = JsonConvert.SerializeObject(value, Formatting.Indented);

            switch (storageType)
            {
                case StorageType.AppSetting:
                    SetJsonSetting(key, json);
                    break;
                case StorageType.LocalFile:
                    await SetJsonFileContent(key, json);
                    break;
            }
        }

        protected abstract void SetJsonSetting(string key, string json);
        protected abstract Task SetJsonFileContent(string file, string json);

        #endregion

        public abstract Task ShowMessage(string message, Exception ex = null);

        public async Task<bool> AddPerson(Person person)
        {
            //TODO write test
            var newId = await AddPersonToData(person);
            if (newId != null)
            {
                person.Id = newId;

                Person.ClearPeopleCache();
                return true;
            }

            return false;
        }
        protected abstract Task<string> AddPersonToData(Person person);

        public abstract Task<List<Person>> GetPeople();

        #region Authentication

        public bool AuthenticateAdmin(string pin)
        {
            return pin == AdminPin;
        }

        public void ChangeAdminPin(string currentPin, string newPin, string newPinConfirm)
        {
            if (currentPin != AdminPin)
            {
                ShowMessage("Incorrect PIN.");
                return;
            }

            if (newPin == AdminPin)
            {
                ShowMessage("The new PIN is the same as the current PIN.");
                return;
            }

            if (newPin.Length != 6)
            {
                ShowMessage("The PIN must be 6 characters long.");
                return;
            }

            if (newPin != newPinConfirm)
            {
                ShowMessage("The confirmation PIN is not the same.");
                return;
            }

            AdminPin = newPin;
        }

        public const string DefaultAdminPin = "123123";
        public string AdminPin
        {
            get
            {
                //Waiting is ok because app settings don't need async
                var savedPin = GetSetting<string>(nameof(AdminPin), StorageType.AppSetting).Result;
                return savedPin ?? DefaultAdminPin;
            }

            private set
            {
                var saveTask = SetSetting(nameof(AdminPin), value, StorageType.AppSetting);
            }
        }

        #endregion
    }

    public enum StorageType
    {
        AppSetting,
        LocalFile
    }
}
